import { test } from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { parseRegister, definedTermSet, indexJson } from "../src/lib/register.js";

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
