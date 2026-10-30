import { test } from "node:test";
import assert from "node:assert/strict";
import { handle, FILES, RAW, LATEST_URL } from "../src/index.js";

const SHA = "e15535a4fbaab894167cc87644baa4168b5f8150";
const req = (path, method = "GET") =>
  new Request(`https://standards.bundu.org${path}`, { method });
const ok = (body, status = 200) => new Response(body, { status });

test("serves a register file at a commit, immutable and open to any origin", async () => {
  let asked;
  const res = await handle(
    req(`/${SHA}/register/register.yaml`),
    async (u) => ((asked = u), ok("standards: []")),
  );
  assert.equal(res.status, 200);
  assert.equal(asked, `${RAW}/${SHA}/register/register.yaml`);
  assert.equal(await res.text(), "standards: []");
  assert.match(res.headers.get("cache-control"), /immutable/);
  assert.equal(res.headers.get("access-control-allow-origin"), "*");
  assert.equal(res.headers.get("x-content-type-options"), "nosniff");
  assert.match(res.headers.get("content-type"), /yaml/);
});

test("serves every allowed file with its own content type", async () => {
  for (const [file, type] of Object.entries(FILES)) {
    const res = await handle(req(`/${SHA}/register/${file}`), async () =>
      ok("x"),
    );
    assert.equal(res.status, 200, file);
    assert.equal(res.headers.get("content-type"), type, file);
  }
});

test("is not an open proxy: only the four register files, only at a full commit sha", async () => {
  const never = async () => assert.fail("must not reach the upstream");
  for (const path of [
    `/${SHA}/register/../AGENTS.md`,
    `/${SHA}/register/%2e%2e/AGENTS.md`,
    `/${SHA}/register/tools/test_register.py`,
    `/${SHA}/register/`,
    `/${SHA}/AGENTS.md`,
    `/main/register/register.yaml`,
    `/${SHA.slice(0, 7)}/register/register.yaml`,
    `/${SHA.toUpperCase()}/register/register.yaml`,
    `/${SHA}/register/__proto__`,
    `/${SHA}/register/constructor`,
    "/etc/passwd",
  ]) {
    const res = await handle(req(path), never);
    assert.equal(res.status, 404, path);
    assert.equal(
      res.headers.get("content-type"),
      "application/problem+json",
      path,
    );
  }
});

test("a missing commit is a 404 problem, an upstream failure a 502", async () => {
  const missing = await handle(
    req(`/${SHA}/register/register.yaml`),
    async () => ok("nope", 404),
  );
  assert.equal(missing.status, 404);
  const down = await handle(req(`/${SHA}/register/register.yaml`), async () =>
    ok("err", 503),
  );
  assert.equal(down.status, 502);
  const thrown = await handle(
    req(`/${SHA}/register/register.yaml`),
    async () => {
      throw new Error("network");
    },
  );
  assert.equal(thrown.status, 502);
  assert.equal(thrown.headers.get("cache-control"), "no-store");
  const body = await thrown.json();
  assert.equal(body.status, 502);
  assert.equal(body.instance, `/${SHA}/register/register.yaml`);
});

test("HEAD has the headers and no body", async () => {
  const res = await handle(
    req(`/${SHA}/register/REGISTER.md`, "HEAD"),
    async () => ok("body"),
  );
  assert.equal(res.status, 200);
  assert.equal(await res.text(), "");
});

test("read-only: other methods are refused, OPTIONS answers the preflight", async () => {
  for (const m of ["POST", "PUT", "DELETE", "PATCH"]) {
    assert.equal(
      (
        await handle(req(`/${SHA}/register/register.yaml`, m), async () =>
          ok("x"),
        )
      ).status,
      405,
      m,
    );
  }
  const pre = await handle(req("/register/latest", "OPTIONS"));
  assert.equal(pre.status, 204);
  assert.match(pre.headers.get("access-control-allow-methods"), /GET/);
});

test("latest names the head of main and the URLs of the four files", async () => {
  let asked;
  const res = await handle(
    req("/register/latest"),
    async (u) => ((asked = u), ok(JSON.stringify({ sha: SHA }))),
  );
  assert.equal(asked, LATEST_URL);
  const body = await res.json();
  assert.equal(body.ref, SHA);
  assert.equal(body.branch, "main");
  assert.equal(body.files.length, Object.keys(FILES).length);
  assert.equal(
    body.files[0].url,
    `https://standards.bundu.org/${SHA}/register/register.yaml`,
  );
  assert.equal(res.headers.get("cache-control"), "public, max-age=300");
});

test("latest refuses an answer that is not a commit", async () => {
  for (const bad of [
    JSON.stringify({ sha: "not-a-sha" }),
    JSON.stringify({}),
    "not json",
  ]) {
    assert.equal(
      (await handle(req("/register/latest"), async () => ok(bad))).status,
      502,
      bad,
    );
  }
  assert.equal(
    (await handle(req("/register/latest"), async () => ok("{}", 403))).status,
    502,
  );
});

test("health and index", async () => {
  assert.equal((await handle(req("/healthz"))).status, 200);
  const idx = await (await handle(req("/"))).json();
  assert.equal(idx.latest, "/register/latest");
});
