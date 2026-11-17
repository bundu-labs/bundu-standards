// Stress tests for the Bundu Cell 0.1.0 draft. Each test records what the draft does
// under a hostile case, so a finding is a test result, not an opinion.
//
//   node --test standards/cell/tools/
//   CELL_PG="psql -h HOST -p PORT -U postgres" node --test standards/cell/tools/   # adds the database round trip

import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { test } from "node:test";
import { canonicalize, hash, keys, parse, render, seal, validate, verifyProof } from "./cell.mjs";

const base = () => ({
  "@context": ["https://schema.org", "https://standards.bundu.org/cell/v0.1"],
  "@type": "MedicalCondition",
  "@id": "urn:cell:01JD3Q7Z8K2V5M",
  name: "Asthma",
  possibleTreatment: { "@type": "Drug", name: "Salbutamol inhaler" },
  dateCreated: "2024-03-11",
  text: "Diagnosed on {{dateCreated}}. Carry the {{possibleTreatment.name}} when walking far.",
  "cell:version": 1,
});
const { privateKey, publicKey } = keys();
const sealed = (c = base()) => seal(c, privateKey);

test("a sealed cell validates, verifies and renders", () => {
  const c = sealed();
  assert.deepEqual(validate(c), { errors: [], warnings: [] });
  assert.ok(verifyProof(c, publicKey));
  assert.equal(render(c), "Diagnosed on 2024-03-11. Carry the Salbutamol inhaler when walking far.");
});

test("changing a fact changes the rendered prose and breaks the old hash", () => {
  const c = sealed();
  const edited = { ...c, possibleTreatment: { "@type": "Drug", name: "Budesonide" } };
  assert.match(validate(edited).errors.join(), /cell:hash does not match/);
  assert.match(render(seal(edited, privateKey)), /Carry the Budesonide/);
});

test("a reference to a missing field is refused", () => {
  const c = sealed({ ...base(), text: "Allergic to {{allergen}}." });
  assert.deepEqual(validate(c).errors, ["text: {{allergen}} names no field"]);
});

test("rule 1 (facts once) is a heuristic: it catches exact copies and misses paraphrase", () => {
  const exact = sealed({ ...base(), text: "Diagnosed on 2024-03-11." });
  assert.match(validate(exact).warnings.join(), /restates dateCreated/);
  const paraphrase = sealed({ ...base(), text: "Diagnosed in March 2024." });
  assert.deepEqual(validate(paraphrase).warnings, [], "the paraphrase slips through");
});

test("a field value that looks like a reference is not expanded twice", () => {
  const c = sealed({ ...base(), name: "{{proof}}", text: "Condition: {{name}}." });
  assert.equal(render(c), "Condition: {{proof}}.");
});

test("key order and whitespace do not change the hash (JCS)", () => {
  const c = sealed();
  const reordered = parse(JSON.stringify(Object.fromEntries(Object.entries(c).reverse()), null, 4));
  assert.equal(hash(reordered), c["cell:hash"]);
});

test("duplicate member names are refused (I-JSON, RFC 7493)", () => {
  assert.throws(() => parse('{"name":"Asthma","name":"Healthy"}'), /duplicate member "name"/);
});

test("unnormalised Unicode changes the hash; NFC is required", () => {
  const nfc = { ...base(), name: "Ché" };
  const nfd = { ...base(), name: "Ché" };
  assert.notEqual(hash(nfc), hash(nfd), "same name on screen, two hashes");
  const c = sealed({ ...base(), text: "Résumé of {{name}}" });
  assert.match(validate(c).errors.join(), /not NFC/);
});

test("integers above 2^53 lose precision in JavaScript, so the hash differs by language", () => {
  const text = '{"@id":"urn:cell:x","nationalId":12345678901234567}';
  const js = canonicalize(JSON.parse(text));
  assert.equal(js, '{"@id":"urn:cell:x","nationalId":12345678901234568}', "JS rounded the number");
  const py = execFileSync("python3", [
    "-c",
    "import json,sys;print(json.dumps(json.loads(sys.argv[1]),separators=(',',':'),sort_keys=True))",
    text,
  ])
    .toString()
    .trim();
  assert.notEqual(py, js, "Python keeps the exact integer; the two canonical forms disagree");
});

test("offline edits on two devices fork the chain: both versions are valid", () => {
  const v1 = sealed();
  const next = (patch) =>
    seal({ ...v1, ...patch, "cell:version": 2, "cell:previous": v1["cell:hash"] }, privateKey);
  const phone = next({ name: "Asthma (mild)" });
  const tablet = next({ dateCreated: "2024-03-12" });
  assert.deepEqual(validate(phone, { previous: v1 }).errors, []);
  assert.deepEqual(validate(tablet, { previous: v1 }).errors, []);
  assert.notEqual(
    phone["cell:hash"],
    tablet["cell:hash"],
    "two different version 2s: the draft has no merge rule",
  );
});

test("deleting an old version's content leaves the chain checkable from hashes alone", () => {
  const v1 = sealed();
  const v2 = seal(
    { ...v1, name: "Asthma (mild)", "cell:version": 2, "cell:previous": v1["cell:hash"] },
    privateKey,
  );
  const tombstone = { "@id": v1["@id"], "cell:version": 1, "cell:hash": v1["cell:hash"] };
  assert.deepEqual(validate(v2, { previous: tombstone }).errors, []);
});

test("text in several languages: a JSON-LD language map, references checked in each", () => {
  const c = sealed({
    ...base(),
    text: { en: "Carry the {{possibleTreatment.name}}.", sn: "Takura {{possibleTreatment.name}} yako." },
  });
  assert.deepEqual(validate(c).errors, []);
  assert.equal(render(c, "sn"), "Takura Salbutamol inhaler yako.");
  const broken = sealed({
    ...base(),
    text: { en: "Carry the {{possibleTreatment.name}}.", sn: "Takura {{mushonga}}." },
  });
  assert.deepEqual(validate(broken).errors, ["text (sn): {{mushonga}} names no field"]);
});

const pg = process.env.CELL_PG;
test(
  "database round trip: jsonb keeps the meaning and the hash, not the bytes",
  { skip: !pg && "set CELL_PG" },
  () => {
    const file = JSON.stringify(sealed({ ...base(), dose: 1.1, "@context": base()["@context"] }), null, 2);
    const psql = (sql) =>
      execFileSync("sh", ["-c", `${pg} -tA -v ON_ERROR_STOP=1`], { input: sql })
        .toString()
        .trim();
    const lit = file.replaceAll("'", "''");
    const asJsonb = psql(`select '${lit}'::jsonb::text;`);
    const asJson = psql(`select '${lit}'::json::text;`);
    assert.equal(asJson, file, "a json column keeps the exact bytes");
    assert.notEqual(asJsonb, file, "a jsonb column rewrites them (key order, spacing)");
    assert.equal(hash(parse(asJsonb)), parse(file)["cell:hash"], "the hash survives jsonb");
    assert.deepEqual(validate(parse(asJsonb)).errors, []);
  },
);
