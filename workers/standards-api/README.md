# standards-api: the file endpoint in Rust

The file endpoint of standards.bundu.org as a Rust Cloudflare Worker (`workers-rs`). It is the
second step of the Deno and Rust pilot (#87). It serves the same routes as
`workers/standards/src/lib/files.js`:

| Route                           | Answer                                                                        |
| ------------------------------- | ----------------------------------------------------------------------------- |
| `/<40-hex commit>/register/<f>` | one of the four register files at that commit, immutable, open to any origin  |
| `/register/latest`              | the head of `main` and the URLs of the four files (REST API, then git's refs) |
| `/healthz`                      | `{"status":"ok"}`                                                             |
| anything else                   | an RFC 9457 problem: 404, or 405 for a method other than GET, HEAD or OPTIONS |

`src/core.rs` holds the behaviour and has no platform dependency. Its tests are the cases of
`workers/standards/test/files.test.js`. `src/lib.rs` is the Worker glue (wasm32 only): `fetch`
to GitHub, and the edge cache keyed on the path without the query string.

## Commands

```sh
cargo test                                               # the 12 cases, natively
cargo clippy --all-targets -- -D warnings
cargo build --release --target wasm32-unknown-unknown
npx wrangler deploy --dry-run                            # runs worker-build; about 181 KiB gzip
```

## Turning it on

Until the binding exists, the Astro Worker answers from its own JS, and nothing changes.

1. Deploy this Worker as `bundu-standards-api`. Either connect it in Workers Builds (root
   directory `workers/standards-api`, deploy command `npx wrangler deploy`; the build needs
   Rust, which `wrangler.toml`'s build command uses), or run `npx wrangler deploy` here once.
2. Add the service binding to `workers/standards/wrangler.jsonc`, in a PR:
   `"services": [{ "binding": "STANDARDS_API", "service": "bundu-standards-api" }]`.
3. On release, the Astro pages `/register/latest` and `/<commit>/register/<file>` hand those
   requests to this Worker. Check that the answers on standards.bundu.org don't change.
4. Once it has run for a while, delete `src/lib/files.js` and its JS tests from
   `workers/standards`.

Checked before the PR (local workerd, `wrangler dev`): all four files at commit `0f859fa`
matched the live JS endpoint byte for byte, as did their content types, `cache-control` and
`etag`. Health, 404, 405, the preflight and `latest` behaved the same as the JS.
