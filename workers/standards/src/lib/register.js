// The register, for the pages. Pure functions: they take the text of
// register/register.yaml, so they run under `node --test` as well as in Astro.
import { parse } from "yaml";

export const SITE = "https://standards.bundu.org";

/**
 * @typedef {{ id: string, name: string, domain: string, category: string, steward?: string,
 *   commitment: string, current: string, governs: string, status: "adopted" | "proposed",
 *   decided: string, evidence?: string[], url: string, docs?: string, notes?: string,
 *   foundational?: boolean }} Standard
 */

/** @param {string} text */
export function parseRegister(text) {
  const doc = parse(text);
  if (!doc || typeof doc.domains !== "object" || !Array.isArray(doc.standards)) {
    throw new Error("register.yaml: expected `domains` and `standards`");
  }
  /** @type {Standard[]} */
  const standards = doc.standards;
  const seen = new Set();
  for (const s of standards) {
    if (!s.id || seen.has(s.id)) throw new Error(`register.yaml: missing or duplicate id ${s.id}`);
    seen.add(s.id);
    if (!(s.domain in doc.domains)) throw new Error(`register.yaml: ${s.id}: unknown domain ${s.domain}`);
    if (s.status !== "adopted" && s.status !== "proposed") throw new Error(`register.yaml: ${s.id}: bad status`);
  }
  const domains = Object.entries(doc.domains)
    .map(([key, title]) => ({ key, title: String(title), standards: standards.filter((s) => s.domain === key) }))
    .filter((d) => d.standards.length > 0);
  const counts = {
    total: standards.length,
    adopted: standards.filter((s) => s.status === "adopted").length,
    proposed: standards.filter((s) => s.status === "proposed").length,
  };
  return { domains, standards, counts };
}

/** schema.org DefinedTerm for one standard. */
export function definedTerm(s, origin = SITE) {
  return {
    "@type": "DefinedTerm",
    "@id": `${origin}/register/${s.id}/`,
    termCode: s.id,
    name: s.name,
    description: s.governs.replace(/\s+/g, " ").trim(),
    url: s.url,
    inDefinedTermSet: `${origin}/register/`,
  };
}

/** schema.org DefinedTermSet for the whole register. */
export function definedTermSet(register, origin = SITE) {
  return {
    "@context": "https://schema.org",
    "@type": "DefinedTermSet",
    "@id": `${origin}/register/`,
    name: "Bundu standards register",
    description: "The external standards the Bundu, Mukoko and Nyuchi platform builds on, and the status of each.",
    url: `${origin}/register/`,
    publisher: { "@type": "Organization", name: "Bundu Foundation", url: "https://www.bundu.org" },
    license: "https://creativecommons.org/publicdomain/zero/1.0/",
    hasDefinedTerm: register.standards.map((s) => definedTerm(s, origin)),
  };
}

/** The machine-readable list at /register/index.json. */
export function indexJson(register, origin = SITE) {
  return {
    count: register.counts,
    standards: register.standards.map((s) => ({
      id: s.id,
      name: s.name,
      domain: s.domain,
      status: s.status,
      commitment: s.commitment,
      current: s.current,
      steward: s.steward ?? null,
      url: s.url,
      page: `${origin}/register/${s.id}/`,
    })),
  };
}

/** The register's two tabs, in order. */
export const TABS = [
  { status: "adopted", label: "Adopted", href: "/register/" },
  { status: "proposed", label: "Proposed", href: "/register/proposed/" },
];

/** The domains, each with only its standards of one status; empty domains dropped. */
export function domainsWith(register, status) {
  return register.domains
    .map((d) => ({ ...d, standards: d.standards.filter((s) => s.status === status) }))
    .filter((d) => d.standards.length > 0);
}

/** The tab a standard is listed under. */
export function tabFor(s) {
  return TABS.find((t) => t.status === s.status) ?? TABS[0];
}

/**
 * A field as paragraphs. In register.yaml a blank line inside a folded scalar starts a new
 * paragraph; YAML turns it into one newline.
 * @param {string | undefined} text
 */
export function paragraphs(text) {
  return (text ?? "")
    .split("\n")
    .map((p) => p.replace(/\s+/g, " ").trim())
    .filter(Boolean);
}

/** The JSON a register tab offers to copy: the same shape as index.json, one status only. */
export function tabJson(register, status, origin = SITE) {
  const all = indexJson(register, origin);
  const standards = all.standards.filter((s) => s.status === status);
  return { status, count: standards.length, standards };
}

/** The JSON a standard's page offers to copy: its whole register record and its page. */
export function standardJson(s, origin = SITE) {
  return { ...s, page: `${origin}/register/${s.id}/` };
}

/**
 * A paragraph as runs of text and `code`: register fields mark code with backticks, as
 * Markdown does. An unpaired backtick stays as text.
 * @param {string} p
 * @returns {{ code: boolean, text: string }[]}
 */
export function codeSpans(p) {
  const parts = p.split("`");
  if (parts.length % 2 === 0) return [{ code: false, text: p }];
  return parts.map((text, i) => ({ code: i % 2 === 1, text })).filter((r) => r.text);
}
