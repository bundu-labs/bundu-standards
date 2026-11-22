# standards-api: standards.bundu.org in Rust

standards.bundu.org as one small native Rust server (axum on tokio): the pages people read and
the file endpoint machines read. It ships as a container (the `Dockerfile` at the repository
root) with no JavaScript runtime, and nothing else at run time: the register, the pages, the
stylesheet and the static files are compiled into the binary.

| Route                                                   | Answer                                                                        |
| ------------------------------------------------------- | ----------------------------------------------------------------------------- |
| `/`, `/register/`, `/register/<id>/`, `/accessibility/` | the pages, rendered with Dioxus and Mzizi Roots from the register             |
| `/register/index.json`                                  | the register as JSON, open to any origin                                      |
| `/<40-hex commit>/register/<f>`                         | one of the four register files at that commit, immutable, open to any origin  |
| `/register/latest`                                      | the head of `main` and the URLs of the four files (REST API, then git's refs) |
| `/healthz`                                              | `{"status":"ok"}`                                                             |
| `/openapi.json`, `/.well-known/openapi.json`            | the OpenAPI 3.0.3 description of every route here (`openapi.json`)            |
| favicons, `robots.txt`, `copy.js`, `ecosystem.jsonld`   | the static files of `public/`, with the rules of `public/_headers`            |
| anything else                                           | an RFC 9457 problem: 404, or 405 for a method other than GET, HEAD or OPTIONS |

- `build.rs` reads `register/register.yaml` at build time, checks it, and writes it into the
  binary as JSON, so a bad register fails the build. It also renders the READMEs of `standards/`
  into pages and lists `public/` for the static files.
- `src/register.rs`, `src/pages.rs` and `src/site.rs` are the pages. A standard's status is the
  Mzizi `Badge` from `mzizi-ui`; the site ships no Tailwind, so the badge's own classes are inert
  and `.chip` (in `styles/site.css`) styles it.
- `src/core.rs` is the file endpoint: it reads four files of this repository from GitHub at a
  commit the caller names, and nothing else (it is not an open proxy). `src/github.rs` is its
  HTTPS client (rustls, no OpenSSL; 5 s to connect, 10 s in all, 3 redirects, 1 MiB at most).
- `src/assets.rs` serves `public/` before anything else and applies `public/_headers` to it:
  the security headers and the content security policy, `ecosystem.jsonld`'s content type and
  CORS. The pages carry the same headers from `src/site.rs`, and a test holds the two policies
  equal. Static files revalidate every time (`max-age=0, must-revalidate`, with an `ETag`).
- `src/server.rs` is the order of answering (static files, the site, the file endpoint) and an
  in-process cache of the file endpoint's successful GETs, for as long as their own
  `cache-control` says (a commit's files for a day at most, `/register/latest` for 5 minutes),
  at most 32 MiB in all and 1 MiB an answer, evicting the soonest to expire; a 404 is kept for a minute so unknown commits do not reach GitHub each time.
- `openapi.json` describes every route above, GET and HEAD only, for API clients. `src/openapi.rs`
  serves it and its tests hold it to the router: a route the server serves (from the register,
  our standards' files, the assets, the path literals in the source, or a link on any page) that
  it leaves out fails `cargo test`, and so does a path in it the server does not serve or serves
  with another content type. Add a route, add it to `openapi.json`.

## Configuration

| Variable        | Default                       | Meaning                                                                    |
| --------------- | ----------------------------- | -------------------------------------------------------------------------- |
| `LISTEN_ADDR`   | `0.0.0.0:8080`                | where to listen                                                            |
| `PUBLIC_ORIGIN` | `https://standards.bundu.org` | the origin the answers name themselves by (the URLs in `/register/latest`) |

`PUBLIC_ORIGIN` is set rather than read from the `Host` header, so a forged header cannot put
another origin into an answer that is then cached. No other setting, and no secret: the server
reads a public repository.

## Commands

```sh
cargo run                                                # http://0.0.0.0:8080 (LISTEN_ADDR to change it)
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo fmt --check
docker build --build-arg BIN=standards -t bundu-standards ../..   # the context is the repository root
docker run --rm -p 8080:8080 bundu-standards
```

## Deploying on Coolify

Coolify builds the root `Dockerfile` (shared with vocab.bundu.org, which is `BIN=vocab`) and runs
the image:

1. New resource, from the public repository `bundu-labs/bundu-standards`, branch `main`.
2. Build pack: **Dockerfile**. Base directory `/`, Dockerfile location `/Dockerfile`. The context
   must be the repository root: the build reads `register/` and `standards/` as well as this
   directory.
3. Ports exposes: `8080`. Build argument `BIN` stays at its default, `standards`. No environment variable is needed; set `PUBLIC_ORIGIN` only if the
   public origin is not `https://standards.bundu.org`.
4. Health check: enable it in Coolify with path `/healthz`, port `8080`, but expect the
   in-container form to fail. The image is distroless, with no shell and no `curl`, and has no
   `HEALTHCHECK` on purpose. Check `GET /healthz` from outside the container (an uptime monitor
   or the proxy's own probe), which answers `{"status":"ok"}`.
5. Domain: `standards.bundu.org` on the Coolify proxy, which terminates TLS. The server speaks
   plain HTTP on 8080 and sets no HSTS; add it at the proxy if wanted.

The image runs as the non-root user (uid 65532) and needs outbound HTTPS to `github.com`,
`api.github.com` and `raw.githubusercontent.com` for `/register/latest` and the commit files;
the pages and static files need nothing.
