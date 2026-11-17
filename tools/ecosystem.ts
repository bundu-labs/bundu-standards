// The ecosystem record's tool: checks ecosystem/ecosystem.yaml and writes what is generated
// from it. `ecosystem.yaml` is the one edit point; the files below are never edited by hand.
//
//   deno run --allow-read --allow-write tools/ecosystem.ts write   # regenerate
//   deno run --allow-read tools/ecosystem.ts check                 # CI: valid and up to date
//
// Generated:
//   ecosystem/ECOSYSTEM.md                 the record as tables, for people
//   ecosystem/ecosystem.jsonld             the record as schema.org JSON-LD, for machines
//   workers/standards/public/ecosystem.jsonld   the same file, served at standards.bundu.org
import { parse } from "jsr:@std/yaml@1.0.9";

export const SITE = "https://standards.bundu.org";
const ROOT = new URL("../", import.meta.url);
const SOURCE = "ecosystem/ecosystem.yaml";
const OUTPUTS = {
  markdown: "ecosystem/ECOSYSTEM.md",
  jsonld: "ecosystem/ecosystem.jsonld",
  served: "workers/standards/public/ecosystem.jsonld",
};

// "proposed": designed, not being built yet (a future build-out). Terms only, never a product.
export const STATUSES = ["canonical", "live", "beta", "in-development", "planned", "proposed", "retired"];
export const PRODUCT_STATUSES = ["live", "beta", "in-development", "planned", "retired"];
export const CONSTRAINTS = [
  "outdoor-readability",
  "connectivity-gradient-design",
  "multilingual-parity",
  "shared-device-awareness",
  "personal-data-sovereignty",
  "no-surveillance-business-model",
  "bundu-floor",
];
const RECORD_FIELDS = ["id", "name", "owner", "version", "status", "last_reviewed", "description"];
const SECTIONS = ["organisations", "pillars", "foundations", "terms", "products"] as const;
type Section = (typeof SECTIONS)[number];

// deno-lint-ignore no-explicit-any
export type Rec = Record<string, any>;
export type Ecosystem = {
  meta: Rec;
  engineering_constraints: Rec[];
  banned_terms: { term: string; use_instead: string }[];
} & Record<Section, Rec[]>;

const DATE = /^\d{4}-\d{2}-\d{2}$/;
/** YAML reads an unquoted 2026-11-17 as a Date; the record means the calendar day. */
export const day = (v: unknown) => (v instanceof Date ? v.toISOString().slice(0, 10) : String(v ?? ""));
const SEMVER = /^\d+\.\d+\.\d+$/;
const ID = /^[a-z0-9]+(-[a-z0-9]+)*$/;

/** Every problem in the record, as messages. Empty means valid. */
export function validate(eco: Ecosystem): string[] {
  const errors: string[] = [];
  const ids = new Set<string>();
  const all: [Section, Rec][] = [];
  for (const section of SECTIONS) {
    if (!Array.isArray(eco[section])) {
      errors.push(`${section}: missing or not a list`);
      continue;
    }
    for (const r of eco[section]) all.push([section, r]);
  }
  for (const [section, r] of all) {
    const where = `${section}/${r.id ?? "?"}`;
    for (const f of RECORD_FIELDS) {
      if (r[f] === undefined || r[f] === "") errors.push(`${where}: missing ${f}`);
    }
    if (r.id !== undefined) {
      if (!ID.test(r.id)) errors.push(`${where}: id must be lowercase words joined by hyphens`);
      if (ids.has(r.id)) errors.push(`${where}: duplicate id`);
      ids.add(r.id);
    }
    if (r.status && !STATUSES.includes(r.status)) errors.push(`${where}: unknown status ${r.status}`);
    if (section === "products") {
      if (r.status && !PRODUCT_STATUSES.includes(r.status)) {
        errors.push(`${where}: a product is live, beta, in-development, planned or retired, not ${r.status}`);
      }
      if (!r.status_source) errors.push(`${where}: missing status_source (where the status was read)`);
    }
    if (section === "pillars" && r.status !== "canonical") errors.push(`${where}: a pillar is canonical`);
    if (r.version && !SEMVER.test(r.version)) errors.push(`${where}: version must be semver`);
    if (r.last_reviewed && !DATE.test(day(r.last_reviewed))) errors.push(`${where}: last_reviewed must be YYYY-MM-DD`);
  }
  if (eco.pillars?.length !== 4) errors.push(`pillars: the ecosystem has four pillars, found ${eco.pillars?.length}`);

  // Every reference resolves to a record in this file (or, for constraints, to the seven).
  const refs = ["owner", "parent", "built_by", "sold_by"];
  for (const [section, r] of all) {
    for (const k of refs) {
      if (r[k] !== undefined && !ids.has(r[k])) errors.push(`${section}/${r.id}: ${k} "${r[k]}" is not a record here`);
    }
    for (const k of ["depends_on", "see"]) {
      for (const t of r[k] ?? []) {
        if (!ids.has(t)) errors.push(`${section}/${r.id}: ${k} "${t}" is not a record here`);
      }
    }
  }
  const constraintIds = (eco.engineering_constraints ?? []).map((c) => c.id);
  if (JSON.stringify(constraintIds) !== JSON.stringify(CONSTRAINTS)) {
    errors.push(`engineering_constraints: must be the seven, in order: ${CONSTRAINTS.join(", ")}`);
  }

  // Banned terms appear in no name, role, audience or description.
  for (const { term, use_instead } of eco.banned_terms ?? []) {
    const re = new RegExp(`\\b${term.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}`, "i");
    for (const [section, r] of all) {
      for (const k of ["name", "role", "audience", "description"]) {
        if (typeof r[k] === "string" && re.test(r[k])) {
          errors.push(`${section}/${r.id}: "${term}" in ${k}; use ${use_instead}`);
        }
      }
    }
  }
  return errors;
}

const iri = (id: string) => `${SITE}/ecosystem#${id}`;
const STATUS_LABEL: Record<string, string> = {
  canonical: "Canonical",
  live: "Live",
  beta: "Beta",
  "in-development": "In development",
  planned: "Planned",
  proposed: "Proposed",
  retired: "Retired",
};

function common(r: Rec): Rec {
  return {
    "@id": iri(r.id),
    identifier: r.id,
    name: r.name,
    description: r.description,
    version: r.version,
    dateModified: day(r.last_reviewed),
    ...(r.url ? { url: r.url } : {}),
  };
}

function org(r: Rec, type = "Organization"): Rec {
  return {
    "@type": type,
    ...common(r),
    ...(r.role ? { slogan: r.role } : {}),
    ...(r.parent ? { parentOrganization: { "@id": iri(r.parent) } } : {}),
  };
}

/** The record as schema.org JSON-LD. */
export function toJsonLd(eco: Ecosystem): Rec {
  const foundation = eco.organisations[0];
  return {
    "@context": "https://schema.org",
    "@id": iri(eco.meta.id),
    "@type": "Dataset",
    name: eco.meta.name,
    version: eco.meta.version,
    dateModified: day(eco.meta.last_reviewed),
    description: eco.meta.governing_principle,
    license: "https://creativecommons.org/publicdomain/zero/1.0/",
    url: `${SITE}/ecosystem.jsonld`,
    publisher: { "@id": iri(foundation.id) },
    hasPart: [
      { ...org(foundation, "NGO"), subOrganization: eco.pillars.map((p) => ({ "@id": iri(p.id) })) },
      ...eco.pillars.map((p) => ({
        ...org(p),
        ...(p.audience ? { audience: { "@type": "Audience", audienceType: p.audience } } : {}),
      })),
      ...eco.foundations.map((f) => ({
        "@type": "CreativeWork",
        ...common(f),
        creativeWorkStatus: STATUS_LABEL[f.status],
        maintainer: { "@id": iri(f.owner) },
        ...(f.built_by ? { creator: { "@id": iri(f.built_by) } } : {}),
      })),
      ...eco.products.map((p) => ({
        "@type": "SoftwareApplication",
        ...common(p),
        creativeWorkStatus: STATUS_LABEL[p.status],
        maintainer: { "@id": iri(p.owner) },
        ...(p.built_by ? { creator: { "@id": iri(p.built_by) } } : {}),
        ...(p.audience ? { audience: { "@type": "Audience", audienceType: p.audience } } : {}),
        ...(p.depends_on ? { softwareRequirements: p.depends_on.map((d: string) => iri(d)) } : {}),
      })),
      {
        "@type": "DefinedTermSet",
        "@id": iri("terms"),
        name: "Bundu ecosystem terms",
        hasDefinedTerm: eco.terms.map((t) => ({
          "@type": "DefinedTerm",
          ...common(t),
          termCode: t.id,
          inDefinedTermSet: { "@id": iri("terms") },
        })),
      },
    ],
  };
}

/** A Markdown table, padded as Prettier pads it, so the generated file passes the repo's lint. */
export function table(head: string[], rows: string[][]): string[] {
  const width = head.map((h, i) => Math.max(3, h.length, ...rows.map((r) => r[i].length)));
  const line = (cells: string[]) => "| " + cells.map((c, i) => c.padEnd(width[i])).join(" | ") + " |";
  return [line(head), "| " + width.map((w) => "-".repeat(w)).join(" | ") + " |", ...rows.map(line)];
}

const cell = (s: unknown) => String(s ?? "").replace(/\s+/g, " ").replace(/\|/g, "\\|").trim();

/** The record as Markdown tables. */
export function toMarkdown(eco: Ecosystem): string {
  const out: string[] = [];
  out.push(
    "# The Bundu ecosystem",
    "",
    "<!-- Generated by tools/ecosystem.ts from ecosystem.yaml. Do not edit: edit ecosystem.yaml and run `deno run --allow-read --allow-write tools/ecosystem.ts write`. -->",
    "",
    `> ${eco.meta.governing_principle}`,
    "",
    `Version ${eco.meta.version}, reviewed ${day(eco.meta.last_reviewed)}. Machine-readable: [${SITE}/ecosystem.jsonld](${SITE}/ecosystem.jsonld).`,
    "",
  );
  const foundation = eco.organisations[0];
  out.push("## The Foundation", "", `**${foundation.name}** (${foundation.legal_form}). ${cell(foundation.description)}`, "");
  out.push("## The four pillars", "");
  out.push(...table(
    ["Pillar", "Role", "Audience", "Built and sold by"],
    eco.pillars.map((p) => {
      const by = [p.built_by && `built by ${name(eco, p.built_by)}`, p.sold_by && `sold by ${name(eco, p.sold_by)}`]
        .filter(Boolean)
        .join(", ");
      return [`**${p.name}**`, cell(p.role), cell(p.audience), by || "—"];
    }),
  ));
  out.push("");
  for (const p of eco.pillars) out.push(`- **${p.name}.** ${cell(p.description)}`);
  out.push("", "## Shared foundations", "", "Foundations enable and connect the pillars. They are not pillars.", "");
  out.push(...table(
    ["Foundation", "Role", "Owner", "Status"],
    eco.foundations.map((f) => [`**${f.name}**`, cell(f.role), name(eco, f.owner), STATUS_LABEL[f.status]]),
  ));
  out.push(
    "",
    "## The seven engineering constraints",
    "",
    "Defined, with their Ubuntu principles and checks, in Mzizi (`get_doctrine`). By id:",
    "",
  );
  eco.engineering_constraints.forEach((c, i) => out.push(`${i + 1}. \`${c.id}\`${c.note ? `: ${c.note}` : ""}`));
  out.push("", "## Terms", "");
  out.push(...table(
    ["Term", "Status", "Definition"],
    eco.terms.map((t) => [`**${t.name}**`, STATUS_LABEL[t.status], cell(t.description)]),
  ));
  out.push("", "Never use: " + eco.banned_terms.map((b) => `"${b.term}" (say ${b.use_instead})`).join("; ") + ".");
  out.push("", "## Products", "");
  const order = (s: string) => PRODUCT_STATUSES.indexOf(s);
  out.push(...table(
    ["Product", "Pillar", "Status", "Audience", "Description"],
    [...eco.products]
      .sort((a, b) => order(a.status) - order(b.status) || a.name.localeCompare(b.name))
      .map((p) => [
        p.url ? `[${p.name}](${p.url})` : p.name,
        name(eco, p.owner),
        STATUS_LABEL[p.status],
        cell(p.audience),
        cell(p.description),
      ]),
  ));
  out.push("", "Statuses are read where `status_source` says, and change here first.", "");
  return out.join("\n");
}

function name(eco: Ecosystem, id: string): string {
  for (const s of SECTIONS) {
    const r = eco[s].find((x) => x.id === id);
    if (r) return r.name;
  }
  return id;
}

export function load(text: string): Ecosystem {
  return parse(text) as Ecosystem;
}

export function render(eco: Ecosystem): Record<keyof typeof OUTPUTS, string> {
  const jsonld = JSON.stringify(toJsonLd(eco), null, 2) + "\n";
  return { markdown: toMarkdown(eco), jsonld, served: jsonld };
}

if (import.meta.main) {
  const mode = Deno.args[0];
  if (mode !== "check" && mode !== "write") {
    console.error("usage: ecosystem.ts check|write");
    Deno.exit(2);
  }
  const eco = load(await Deno.readTextFile(new URL(SOURCE, ROOT)));
  const errors = validate(eco);
  if (errors.length) {
    console.error(`${SOURCE} is not valid:\n` + errors.map((e) => `  - ${e}`).join("\n"));
    Deno.exit(1);
  }
  const files = render(eco);
  const stale: string[] = [];
  for (const [k, path] of Object.entries(OUTPUTS) as [keyof typeof OUTPUTS, string][]) {
    const url = new URL(path, ROOT);
    let current = "";
    try {
      current = await Deno.readTextFile(url);
    } catch { /* missing */ }
    if (current === files[k]) continue;
    if (mode === "write") await Deno.writeTextFile(url, files[k]);
    else stale.push(path);
  }
  if (stale.length) {
    console.error(`Out of date: ${stale.join(", ")}. Run: deno run --allow-read --allow-write tools/ecosystem.ts write`);
    Deno.exit(1);
  }
  console.log(mode === "write" ? "ecosystem: written" : "ecosystem: valid and up to date");
}
