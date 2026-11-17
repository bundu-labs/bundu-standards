import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { parseRegister, definedTermSet, indexJson, domainsWith, paragraphs, tabJson, standardJson, tabFor, codeSpans } from "../src/lib/register.js";

const text = readFileSync(new URL("../../../register/register.yaml", import.meta.url), "utf8");
const reg = parseRegister(text);

test("the real register parses and every standard has a page-able id", () => {
  assert.ok(reg.counts.total > 0);
  assert.equal(reg.counts.adopted + reg.counts.proposed, reg.counts.total);
  for (const s of reg.standards) assert.match(s.id, /^[a-z0-9_]+$/, s.id);
});

test("every standard appears in exactly one domain", () => {
  const n = reg.domains.reduce((a, d) => a + d.standards.length, 0);
  assert.equal(n, reg.counts.total);
});

test("rejects duplicate ids, unknown domains and bad status", () => {
  const base = "domains:\n  a: A\nstandards:\n";
  const one = (extra) => `${base}  - id: x\n    domain: a\n    status: adopted\n${extra}`;
  assert.throws(() => parseRegister(`${one("")}  - id: x\n    domain: a\n    status: adopted\n`), /duplicate/);
  assert.throws(() => parseRegister(`${base}  - id: x\n    domain: b\n    status: adopted\n`), /unknown domain/);
  assert.throws(() => parseRegister(`${base}  - id: x\n    domain: a\n    status: maybe\n`), /bad status/);
});

test("JSON-LD and index list every standard", () => {
  assert.equal(definedTermSet(reg).hasDefinedTerm.length, reg.counts.total);
  const idx = indexJson(reg);
  assert.equal(idx.standards.length, reg.counts.total);
  assert.ok(idx.standards.every((s) => s.page.startsWith("https://standards.bundu.org/register/")));
});

test("the two tabs split the register by status, and no standard's page collides with a tab", () => {
  const n = (status) => domainsWith(reg, status).reduce((a, d) => a + d.standards.length, 0);
  assert.equal(n("adopted"), reg.counts.adopted);
  assert.equal(n("proposed"), reg.counts.proposed);
  assert.equal(tabJson(reg, "proposed").count, reg.counts.proposed);
  assert.ok(!reg.standards.some((s) => s.id === "proposed"), "/register/proposed/ is the tab");
  assert.equal(tabFor({ status: "proposed" }).href, "/register/proposed/");
});

test("a blank line in a field starts a new paragraph", () => {
  assert.deepEqual(paragraphs("One.  Two\n Three."), ["One. Two", "Three."]);
  assert.deepEqual(paragraphs(undefined), []);
});

test("a standard's page offers its whole record as JSON", () => {
  const s = reg.standards[0];
  const j = standardJson(s);
  assert.equal(j.id, s.id);
  assert.equal(j.decided, s.decided);
  assert.equal(j.page, `https://standards.bundu.org/register/${s.id}/`);
});

test("backticks in a field become code, and an unpaired one stays text", () => {
  assert.deepEqual(codeSpans("pin `v1` now"), [
    { code: false, text: "pin " },
    { code: true, text: "v1" },
    { code: false, text: " now" },
  ]);
  assert.deepEqual(codeSpans("a ` b"), [{ code: false, text: "a ` b" }]);
});
