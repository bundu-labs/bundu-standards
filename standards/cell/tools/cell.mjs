// Bundu Cell 0.1.0 draft: a prototype validator. It exists to stress-test the draft,
// not to be the reference implementation. Plain ESM with node: built-ins, so it runs
// under Node (`node --test`) and Deno alike.
//
//   node standards/cell/tools/cell.mjs check FILE.cell    validate one cell
//   node standards/cell/tools/cell.mjs render FILE.cell   print its text with facts filled in

import { createHash, generateKeyPairSync, sign, verify } from "node:crypto";
import { readFileSync } from "node:fs";

/** RFC 8785 (JCS) canonical form. Numbers use the ECMAScript form, which JCS adopts. */
export function canonicalize(v) {
  if (v === null || typeof v !== "object") {
    if (typeof v === "number" && !Number.isFinite(v)) throw new Error("JCS: not a finite number");
    return JSON.stringify(v);
  }
  if (Array.isArray(v)) return `[${v.map(canonicalize).join(",")}]`;
  // JCS sorts keys by UTF-16 code units, which is what the default sort compares.
  const keys = Object.keys(v)
    .filter((k) => v[k] !== undefined)
    .sort();
  return `{${keys.map((k) => `${JSON.stringify(k)}:${canonicalize(v[k])}`).join(",")}}`;
}

const ENVELOPE = ["cell:hash", "proof"];

/** The content: the cell without its hash and proof. */
export function content(cell) {
  return Object.fromEntries(Object.entries(cell).filter(([k]) => !ENVELOPE.includes(k)));
}

export function hash(cell) {
  return `sha256-${createHash("sha256")
    .update(canonicalize(content(cell)), "utf8")
    .digest("base64url")}`;
}

/** I-JSON (RFC 7493): refuse duplicate member names, which JSON.parse silently drops. */
export function parse(text) {
  const dup = findDuplicateKey(text);
  if (dup) throw new Error(`I-JSON: duplicate member "${dup}"`);
  return JSON.parse(text);
}

function findDuplicateKey(text) {
  // Walks the JSON once, keeping a stack of open containers and each object's keys.
  const stack = [];
  let expectKey = false;
  for (let i = 0; i < text.length; i++) {
    const c = text[i];
    if (c === '"') {
      let j = i + 1;
      while (text[j] !== '"') j += text[j] === "\\" ? 2 : 1;
      if (expectKey) {
        const keys = stack[stack.length - 1].keys;
        const k = JSON.parse(text.slice(i, j + 1));
        if (keys.has(k)) return k;
        keys.add(k);
        expectKey = false;
      }
      i = j;
    } else if (c === "{") {
      stack.push({ keys: new Set() });
      expectKey = true;
    } else if (c === "[") stack.push({ keys: null });
    else if (c === "}" || c === "]") stack.pop();
    else if (c === ",") expectKey = stack[stack.length - 1]?.keys != null;
  }
  return null;
}

const REF = /\{\{\s*([^{}\s]+)\s*\}\}/g;

/** Resolve a dotted path; a numeric segment indexes an array. */
export function resolve(cell, path) {
  let v = cell;
  for (const seg of path.split(".")) {
    if (v === null || typeof v !== "object") return undefined;
    v = Array.isArray(v) && /^\d+$/.test(seg) ? v[Number(seg)] : v[seg];
  }
  return v;
}

/** The text in every language: a string, or a JSON-LD language map. */
export function texts(cell) {
  const t = cell.text;
  if (t === undefined) return {};
  if (typeof t === "string") return { "": t };
  if (t && typeof t === "object" && !Array.isArray(t)) return t;
  throw new Error("text must be a string or a language map");
}

/** Fill references in one pass: a value that itself contains {{...}} is not expanded again. */
export function render(cell, lang = "") {
  const all = texts(cell);
  const t = all[lang] ?? Object.values(all)[0] ?? "";
  return t.replace(REF, (_, p) => {
    const v = resolve(cell, p);
    return typeof v === "object" ? JSON.stringify(v) : String(v);
  });
}

/** Scalar facts worth looking for in prose: strings of 4+ characters, numbers, years. */
function facts(v, path = "", out = []) {
  if (v === null) return out;
  if (typeof v === "object") {
    for (const [k, x] of Object.entries(v)) {
      if (k.startsWith("@") || k.startsWith("cell:") || k === "text" || k === "proof") continue;
      facts(x, path ? `${path}.${k}` : k, out);
    }
  } else if (typeof v === "number" || (typeof v === "string" && v.length >= 4)) {
    out.push([path, String(v)]);
  }
  return out;
}

export function validate(cell, { previous } = {}) {
  const errors = [];
  const warnings = [];
  for (const k of ["@context", "@type", "@id", "cell:version", "cell:hash"]) {
    if (cell[k] === undefined) errors.push(`missing ${k}`);
  }
  let all = {};
  try {
    all = texts(cell);
  } catch (e) {
    errors.push(e.message);
  }
  for (const [lang, t] of Object.entries(all)) {
    const at = lang ? ` (${lang})` : "";
    if (t.normalize("NFC") !== t) errors.push(`text${at} is not NFC`);
    for (const [, p] of t.matchAll(REF)) {
      if (resolve(cell, p) === undefined) errors.push(`text${at}: {{${p}}} names no field`);
    }
    // Rule 1, heuristically: a fact's value written out in prose instead of referenced.
    const prose = t.replace(REF, "");
    for (const [p, v] of facts(content(cell))) {
      if (prose.includes(v)) warnings.push(`text${at} restates ${p} ("${v}"); write {{${p}}}`);
    }
  }
  if (cell["cell:hash"] !== undefined && cell["cell:hash"] !== hash(cell)) {
    errors.push("cell:hash does not match the content");
  }
  if (previous) {
    if (cell["@id"] !== previous["@id"]) errors.push("a new version keeps the @id");
    if (cell["cell:version"] !== previous["cell:version"] + 1) errors.push("cell:version must go up by one");
    if (cell["cell:previous"] !== previous["cell:hash"])
      errors.push("cell:previous must be the hash of the version before");
  }
  return { errors, warnings };
}

/** Seal: set the hash, then sign the canonical content plus the hash. Prototype proof only. */
export function seal(cell, privateKey) {
  const out = { ...content(cell) };
  out["cell:hash"] = hash(out);
  const sig = sign(null, Buffer.from(canonicalize(out)), privateKey);
  out.proof = {
    type: "DataIntegrityProof",
    cryptosuite: "eddsa-jcs-2022",
    proofValue: `u${sig.toString("base64url")}`,
  };
  return out;
}

export function verifyProof(cell, publicKey) {
  const { proof, ...rest } = cell;
  if (!proof) return false;
  return verify(
    null,
    Buffer.from(canonicalize(rest)),
    publicKey,
    Buffer.from(proof.proofValue.slice(1), "base64url"),
  );
}

export const keys = () => generateKeyPairSync("ed25519");

if (import.meta.main ?? (process.argv[1] && import.meta.url.endsWith(process.argv[1].split("/").pop()))) {
  const [cmd, file] = process.argv.slice(2);
  const cell = parse(readFileSync(file, "utf8"));
  if (cmd === "render") console.log(render(cell));
  else {
    const { errors, warnings } = validate(cell);
    for (const w of warnings) console.log(`warning: ${w}`);
    for (const e of errors) console.log(`error: ${e}`);
    process.exitCode = errors.length ? 1 : 0;
  }
}
