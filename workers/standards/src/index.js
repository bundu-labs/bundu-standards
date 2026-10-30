// standards.bundu.org: the public, read-only address of the Bundu standards register.
//
// It holds no data and no secret. It serves four files of `register/` from the
// public repository `bundu-labs/bundu-standards` at a commit the caller names, so:
//
//   GET /<40-hex commit>/register/register.yaml
//   GET /<40-hex commit>/register/REGISTER.md
//   GET /<40-hex commit>/register/seed/integrations.standards.json
//   GET /<40-hex commit>/register/tools/register.py
//   GET /register/latest            the head of `main`, as JSON
//   GET /healthz
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

const COMMON = {
  "access-control-allow-origin": "*",
  "x-content-type-options": "nosniff",
  "referrer-policy": "no-referrer",
};

/** RFC 9457 problem details. */
export function problem(status, title, detail, instance) {
  return new Response(
    JSON.stringify({ type: "about:blank", title, status, detail, instance }),
    {
      status,
      headers: {
        ...COMMON,
        "content-type": "application/problem+json",
        "cache-control": "no-store",
      },
    },
  );
}

function json(body, status = 200, cache = "no-store") {
  return new Response(JSON.stringify(body), {
    status,
    headers: {
      ...COMMON,
      "content-type": "application/json; charset=utf-8",
      "cache-control": cache,
    },
  });
}

/**
 * Handle one request. `fetchImpl` is injected so the whole thing is testable
 * without a network.
 */
export async function handle(request, fetchImpl = fetch) {
  const url = new URL(request.url);
  const path = url.pathname;

  if (request.method === "OPTIONS") {
    return new Response(null, {
      status: 204,
      headers: {
        ...COMMON,
        "access-control-allow-methods": "GET, HEAD, OPTIONS",
        "access-control-max-age": "86400",
      },
    });
  }
  if (request.method !== "GET" && request.method !== "HEAD") {
    return problem(
      405,
      "Method not allowed",
      "This endpoint is read-only.",
      path,
    );
  }

  if (path === "/healthz") return json({ status: "ok" });

  if (path === "/") {
    return json(
      {
        name: "Bundu standards register",
        repository: `https://github.com/${REPO}`,
        files: Object.keys(FILES).map((f) => `/<commit>/register/${f}`),
        latest: "/register/latest",
        licence: "Apache-2.0 (code), CC0 (vocabulary)",
      },
      200,
      "public, max-age=300",
    );
  }

  if (path === "/register/latest") return latest(url, fetchImpl, path);

  const m = SHA_PATH.exec(path);
  if (!m) return problem(404, "Not found", "No such resource.", path);
  const [, sha, file] = m;
  const type = Object.hasOwn(FILES, file) ? FILES[file] : undefined;
  if (!type)
    return problem(
      404,
      "Not found",
      "That file is not part of the register.",
      path,
    );

  let upstream;
  try {
    upstream = await fetchImpl(`${RAW}/${sha}/register/${file}`, {
      headers: { "user-agent": "standards.bundu.org" },
    });
  } catch {
    return problem(
      502,
      "Bad gateway",
      "The repository could not be reached.",
      path,
    );
  }
  if (upstream.status === 404)
    return problem(
      404,
      "Not found",
      "That commit or file does not exist.",
      path,
    );
  if (!upstream.ok)
    return problem(
      502,
      "Bad gateway",
      `The repository answered ${upstream.status}.`,
      path,
    );

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
      headers: {
        accept: "application/vnd.github+json",
        "user-agent": "standards.bundu.org",
      },
    });
  } catch {
    return problem(
      502,
      "Bad gateway",
      "The repository could not be reached.",
      path,
    );
  }
  if (!res.ok)
    return problem(
      502,
      "Bad gateway",
      `The repository answered ${res.status}.`,
      path,
    );
  let body;
  try {
    body = await res.json();
  } catch {
    return problem(
      502,
      "Bad gateway",
      "The repository answer was not JSON.",
      path,
    );
  }
  const ref = body && typeof body.sha === "string" ? body.sha : "";
  if (!SHA.test(ref))
    return problem(
      502,
      "Bad gateway",
      "The repository answer had no commit.",
      path,
    );
  return json(
    {
      ref,
      branch: "main",
      files: Object.keys(FILES).map((f) => ({
        path: f,
        url: `${url.origin}/${ref}/register/${f}`,
      })),
    },
    200,
    "public, max-age=300",
  );
}

export default {
  async fetch(request, _env, ctx) {
    // Cache successful GETs at the edge. A sha URL is immutable; "latest" is 5 minutes.
    const cache = typeof caches !== "undefined" ? caches.default : null;
    if (cache && request.method === "GET") {
      const hit = await cache.match(request);
      if (hit) return hit;
    }
    const res = await handle(request);
    if (cache && request.method === "GET" && res.status === 200 && ctx) {
      ctx.waitUntil(cache.put(request, res.clone()));
    }
    return res;
  },
};
