// The file endpoint of standards.bundu.org.
//
// It holds no data and no secret. It serves four files of `register/` from the
// public repository `bundu-labs/bundu-standards` at a commit the caller names:
//
//   /<40-hex commit>/register/register.yaml
//   /<40-hex commit>/register/REGISTER.md
//   /<40-hex commit>/register/seed/integrations.standards.json
//   /<40-hex commit>/register/tools/register.py
//   /register/latest            the head of `main`, as JSON
//   /healthz
//
// The path after the host is the same as on raw.githubusercontent.com, so a
// client can switch between the two by changing only its base URL.
//
// A commit is immutable, so those answers are cached for a year. A caller who
// needs more than that pins the sha256 of each file as well (see standards.lock
// in a consumer): the endpoint is a convenience and an address that survives a
// repository move, never the thing a consumer has to trust.

export const REPO = "bundu-labs/bundu-standards";
export const RAW = `https://raw.githubusercontent.com/${REPO}`;
export const LATEST_URL = `https://api.github.com/repos/${REPO}/commits/main`;

/** The only files served. Anything else is a 404: this is not an open proxy. */
export const FILES = {
  "register.yaml": "application/yaml; charset=utf-8",
  "REGISTER.md": "text/markdown; charset=utf-8",
  "seed/integrations.standards.json": "application/json; charset=utf-8",
  "tools/register.py": "text/x-python; charset=utf-8",
};

const SHA = /^[0-9a-f]{40}$/;
const SHA_PATH = /^\/([0-9a-f]{40})\/register\/(.+)$/;

export const COMMON = {
  "access-control-allow-origin": "*",
  "x-content-type-options": "nosniff",
  "referrer-policy": "no-referrer",
};

/** RFC 9457 problem details. */
export function problem(status, title, detail, instance) {
  return new Response(JSON.stringify({ type: "about:blank", title, status, detail, instance }), {
    status,
    headers: { ...COMMON, "content-type": "application/problem+json", "cache-control": "no-store" },
  });
}

function json(body, status = 200, cache = "no-store") {
  return new Response(JSON.stringify(body), {
    status,
    headers: { ...COMMON, "content-type": "application/json; charset=utf-8", "cache-control": cache },
  });
}

/**
 * Answer one request for /healthz, /register/latest or /<commit>/register/<file>.
 * `fetchImpl` is injected so the whole thing is testable without a network.
 */
export async function route(request, fetchImpl = fetch) {
  const url = new URL(request.url);
  const path = url.pathname;

  if (request.method === "OPTIONS") {
    return new Response(null, {
      status: 204,
      headers: { ...COMMON, "access-control-allow-methods": "GET, HEAD, OPTIONS", "access-control-max-age": "86400" },
    });
  }
  if (request.method !== "GET" && request.method !== "HEAD") {
    return problem(405, "Method not allowed", "This endpoint is read-only.", path);
  }

  if (path === "/healthz" || path === "/healthz/") return json({ status: "ok" });
  if (path === "/register/latest" || path === "/register/latest/") return latest(url, fetchImpl, path);

  const m = SHA_PATH.exec(path);
  if (!m) return problem(404, "Not found", "No such resource.", path);
  const [, sha, file] = m;
  const type = Object.hasOwn(FILES, file) ? FILES[file] : undefined;
  if (!type) return problem(404, "Not found", "That file is not part of the register.", path);

  let upstream;
  try {
    upstream = await fetchImpl(`${RAW}/${sha}/register/${file}`, { headers: { "user-agent": "standards.bundu.org" } });
  } catch {
    return problem(502, "Bad gateway", "The repository could not be reached.", path);
  }
  if (upstream.status === 404) return problem(404, "Not found", "That commit or file does not exist.", path);
  if (!upstream.ok) return problem(502, "Bad gateway", `The repository answered ${upstream.status}.`, path);

  return new Response(request.method === "HEAD" ? null : upstream.body, {
    status: 200,
    headers: {
      ...COMMON,
      "content-type": type,
      // A commit never changes.
      "cache-control": "public, max-age=31536000, immutable",
      etag: `"${sha}:${file}"`,
    },
  });
}

async function latest(url, fetchImpl, path) {
  let res;
  try {
    res = await fetchImpl(LATEST_URL, {
      headers: { accept: "application/vnd.github+json", "user-agent": "standards.bundu.org" },
    });
  } catch {
    return problem(502, "Bad gateway", "The repository could not be reached.", path);
  }
  if (!res.ok) return problem(502, "Bad gateway", `The repository answered ${res.status}.`, path);
  let body;
  try {
    body = await res.json();
  } catch {
    return problem(502, "Bad gateway", "The repository answer was not JSON.", path);
  }
  const ref = body && typeof body.sha === "string" ? body.sha : "";
  if (!SHA.test(ref)) return problem(502, "Bad gateway", "The repository answer had no commit.", path);
  return json(
    {
      ref,
      branch: "main",
      files: Object.keys(FILES).map((f) => ({ path: f, url: `${url.origin}/${ref}/register/${f}` })),
    },
    200,
    "public, max-age=300",
  );
}

/** The cache key ignores the query string: `?x=1`, `?x=2` must not each reach the upstream. */
function cacheKey(request) {
  const url = new URL(request.url);
  return new Request(`${url.origin}${url.pathname}`, { method: "GET" });
}

/** `route`, with successful GETs kept in the edge cache when there is one. */
export async function routeCached(request, fetchImpl = fetch, cache = globalThis.caches?.default) {
  const key = cache && request.method === "GET" ? cacheKey(request) : null;
  if (key) {
    const hit = await cache.match(key);
    if (hit) return hit;
  }
  const res = await route(request, fetchImpl);
  if (key && res.status === 200) {
    await cache.put(key, res.clone());
  }
  return res;
}
