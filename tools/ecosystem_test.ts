import { assert, assertEquals } from "jsr:@std/assert@1.0.13";
import { CONSTRAINTS, type Ecosystem, load, PRODUCT_STATUSES, render, toJsonLd, validate } from "./ecosystem.ts";

const real = () => load(Deno.readTextFileSync(new URL("../ecosystem/ecosystem.yaml", import.meta.url)));

Deno.test("the real record is valid", () => {
  assertEquals(validate(real()), []);
});

Deno.test("four pillars, the seven constraints in order", () => {
  const eco = real();
  assertEquals(eco.pillars.map((p) => p.id), ["bundu-labs", "nyuchi-africa", "mukoko", "shamwari-ai"]);
  assertEquals(eco.engineering_constraints.map((c) => c.id), CONSTRAINTS);
});

Deno.test("a duplicate id, a dangling reference and a bad status are refused", () => {
  const eco: Ecosystem = structuredClone(real());
  eco.products.push({ ...eco.products[0] });
  eco.products[1].owner = "nobody";
  eco.products[2].status = "canonical";
  const errors = validate(eco).join("\n");
  assert(errors.includes("duplicate id"), errors);
  assert(errors.includes('owner "nobody" is not a record here'), errors);
  assert(errors.includes("a product is live, beta, in-development, planned or retired"), errors);
});

Deno.test("a banned term in a description is refused", () => {
  const eco: Ecosystem = structuredClone(real());
  eco.terms[0].description = "Your data in a Web3 pod.";
  assert(validate(eco).some((e) => e.includes('"Web3"') && e.includes("use pod")));
});

Deno.test("a fifth pillar or a missing constraint is refused", () => {
  const eco: Ecosystem = structuredClone(real());
  eco.pillars.push({ ...eco.pillars[0], id: "extra" });
  eco.engineering_constraints.pop();
  const errors = validate(eco).join("\n");
  assert(errors.includes("four pillars"), errors);
  assert(errors.includes("must be the seven"), errors);
});

Deno.test("JSON-LD: the Foundation holds the four pillars; products carry status and owner", () => {
  const ld = toJsonLd(real());
  const parts = ld.hasPart as Record<string, unknown>[];
  const foundation = parts.find((p) => p.identifier === "bundu-foundation")!;
  assertEquals((foundation.subOrganization as unknown[]).length, 4);
  const api = parts.find((p) => p.identifier === "mukoko-weather")!;
  assertEquals(api.creativeWorkStatus, "Live");
  assertEquals(api.maintainer, { "@id": "https://standards.bundu.org/ecosystem#mukoko" });
});

Deno.test("the served copy is the published JSON-LD", () => {
  const files = render(real());
  assertEquals(files.served, files.jsonld);
  assert(files.markdown.startsWith("# The Bundu ecosystem"));
});

Deno.test("the platform tokens are proposed, never more", () => {
  const t = real().terms.find((x) => x.id === "platform-tokens")!;
  assertEquals(t.status, "proposed");
  assert(!PRODUCT_STATUSES.includes("proposed"), "proposed is not a product status");
});
