// The vocabulary contract, checked against a running vocab server.
//
// Term URIs are a permanent contract: every published term must resolve as JSON-LD and as an
// HTML page, the default representation is JSON-LD, and an unknown term is a 404. CI starts the
// release build and runs this; it replaces the inline Python and curl checks it had before.
//
// Usage: deno run --allow-net tools/contract.ts [base-url]   (default http://127.0.0.1:8080)

const BASE = (Deno.args[0] ?? "http://127.0.0.1:8080").replace(/\/$/, "");
const errors: string[] = [];
const fail = (msg: string) => {
  errors.push(msg);
  console.log(`::error::${msg}`);
};

async function get(path: string, accept?: string): Promise<Response> {
  return await fetch(
    `${BASE}${path}`,
    accept ? { headers: { Accept: accept } } : undefined,
  );
}

const contextRes = await get("/context.jsonld");
if (!contextRes.ok) {
  console.log(`::error::/context.jsonld answered ${contextRes.status}`);
  Deno.exit(1);
}
const context = (await contextRes.json())["@context"] as Record<
  string,
  unknown
>;
const terms = Object.keys(context).filter(
  (k) => k !== "schema" && k !== "bundu",
);
console.log(`checking ${terms.length} terms: ${terms.join(" ")}`);

for (const t of terms) {
  const ld = await get(`/${t}`, "application/ld+json");
  if (!ld.ok) {
    fail(`${t} does not resolve as JSON-LD (${ld.status})`);
    await ld.body?.cancel();
    continue;
  }
  const d = await ld.json();
  if (d["@id"] !== `bundu:${t}`)
    fail(`${t}: @id is ${d["@id"]}, want bundu:${t}`);
  if (!["stable", "testing"].includes(d["vs:term_status"])) {
    fail(
      `${t}: vs:term_status is ${d["vs:term_status"]}, want stable or testing`,
    );
  }
  if (!d["rdfs:comment"] || !d["bundu:rationale"])
    fail(`${t}: missing rdfs:comment or bundu:rationale`);

  const html = await get(`/${t}`, "text/html");
  if (!html.ok) {
    fail(`${t} does not resolve as HTML (${html.status})`);
    await html.body?.cancel();
    continue;
  }
  if (!(await html.text()).includes(`<code class="term">${t}</code>`)) {
    fail(`${t} HTML page does not render the term`);
  }
}

// Content negotiation and 404 behaviour.
const mediaType = (r: Response) =>
  (r.headers.get("content-type") ?? "").split(";")[0].trim();
const byDefault = await get("/Family");
await byDefault.body?.cancel();
if (mediaType(byDefault) !== "application/ld+json") {
  fail(
    `default representation must be JSON-LD, got ${byDefault.headers.get("content-type")}`,
  );
}
const asHtml = await get("/Family", "text/html");
await asHtml.body?.cancel();
if (!mediaType(asHtml).startsWith("text/html")) {
  fail(`HTML not served, got ${asHtml.headers.get("content-type")}`);
}
const unknown = await get("/NotATerm");
await unknown.body?.cancel();
if (unknown.status !== 404) fail(`unknown term returned ${unknown.status}`);

if (errors.length) {
  console.log(`${errors.length} contract failure(s)`);
  Deno.exit(1);
}
console.log(
  `contract holds: ${terms.length} terms in both representations, negotiation and 404 correct`,
);
