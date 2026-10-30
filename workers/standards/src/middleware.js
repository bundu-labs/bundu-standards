// Security headers for the pages the Worker renders. Static pages get theirs
// from public/_headers (Cloudflare does not apply a Worker's headers to assets).
export async function onRequest(_context, next) {
  const res = await next();
  const h = new Headers(res.headers);
  h.set("x-content-type-options", "nosniff");
  h.set("referrer-policy", "no-referrer");
  h.set("x-frame-options", "DENY");
  return new Response(res.body, { status: res.status, statusText: res.statusText, headers: h });
}
