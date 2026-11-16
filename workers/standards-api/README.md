# standards-api: standards.bundu.org in Rust

standards.bundu.org as one Rust Cloudflare Worker (`workers-rs`): the pages people read and the
file endpoint machines read. It is the Deno and Rust pilot (#87), and it replaces the Astro
Worker in `workers/standards` once the custom domain moves here.

| Route                                                   | Answer                                                                        |
| ------------------------------------------------------- | ----------------------------------------------------------------------------- |
| `/`, `/register/`, `/register/<id>/`, `/accessibility/` | the pages, rendered with Dioxus and Mzizi Roots from the register             |
| `/register/index.json`                                  | the register as JSON, open to any origin                                      |
| `/<40-hex commit>/register/<f>`                         | one of the four register files at that commit, immutable, open to any origin  |
| `/register/latest`                                      | the head of `main` and the URLs of the four files (REST API, then git's refs) |
| `/healthz`                                              | `{"status":"ok"}`                                                             |
| favicons, `robots.txt`                                  | static assets from `../standards/public`, with its `_headers`                 |
| anything else                                           | an RFC 9457 problem: 404, or 405 for a method other than GET, HEAD or OPTIONS |

- `build.rs` reads `register/register.yaml` at build time, checks it, and writes it into the
  Worker as JSON, so a bad register fails the build and no YAML parser ships in the wasm.
- `src/register.rs`, `src/pages.rs` and `src/site.rs` are the pages: the Rust port of
  `register.js`, `Base.astro` and its four pages. Same markup, same classes and text, styled by
  the same `site.css`. A standard's status is the Mzizi `Badge` from `mzizi-ui`; the site ships
  no Tailwind, so the badge's own classes are inert and `.chip` styles it.
- `src/core.rs` is the file endpoint. Its tests are the cases of
  `workers/standards/test/files.test.js`. `src/lib.rs` is the Worker glue (wasm32 only).

All of it has no platform dependency apart from `lib.rs`, and is tested natively.

## Commands

```sh
cargo test                                               # natively
cargo clippy --all-targets -- -D warnings
cargo build --release --target wasm32-unknown-unknown
npx wrangler deploy --dry-run                            # runs worker-build; about 340 KiB gzip
```

## Turning it on

Until the custom domain moves, the Astro Worker serves standards.bundu.org and nothing changes.

1. Deploy this Worker as `bundu-standards-api`. Either connect it in Workers Builds (root
   directory `workers/standards-api`, deploy command `npx wrangler deploy`; the build needs
   Rust, which `wrangler.toml`'s build command uses), or run `npx wrangler deploy` here once.
2. Check it on its preview URL against standards.bundu.org.
3. Move the custom domain `standards.bundu.org` from `bundu-standards` to
   `bundu-standards-api` (Workers, Settings, Domains & Routes), then add
   `routes = [{ pattern = "standards.bundu.org", custom_domain = true }]` to `wrangler.toml`
   in a PR, so the next deploy keeps it.
4. Once it has run for a while, delete `workers/standards` in a PR, and move `public/` and
   `site.css` here.

Checked before the PR:

- All 108 pages and `/register/index.json` match the Astro build (`astro build` at the same
  commit). The JSON is byte for byte the same; the HTML has the same elements, attributes and
  text. The one difference is a fix: the home page now reads "103 adopted, 2 proposed", where
  Astro dropped the space after the comma.
- On local workerd (`wrangler dev`): every page answers 200 with the security headers from
  `_headers`, the assets are served, and the file endpoint behaves as before.
- Earlier (#90): the four files at commit `0f859fa` matched the live JS endpoint byte for
  byte, as did their content types, `cache-control` and `etag`.
