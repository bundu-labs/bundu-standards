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
