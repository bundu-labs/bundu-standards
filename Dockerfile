# One build for both sites. Coolify runs two apps from this repository and this file; they
# differ only in the BIN build argument and the domain:
#
#   BIN=standards  (the default)  standards.bundu.org  workers/standards-api
#   BIN=vocab                     vocab.bundu.org      the root crate (src/)
#
# Both render their pages with the shared shell and stylesheet in site/ (bundu-site).
#
# Build from the repository root. The standards build reads register/ and standards/ as well as
# its crate (build.rs), so they are in the context (.dockerignore keeps the rest out). Nothing
# is read at run time: everything the server serves is inside the binary.
#
# Both listen on LISTEN_ADDR (default 0.0.0.0:8080), answer GET /healthz with {"status":"ok"},
# and run as the distroless non-root user. The runtime image has no shell and no curl, so the
# HEALTHCHECK runs the binary itself: `/server healthcheck` does a local GET of /healthz and exits
# 0 on a 200 (exec form, no shell). Coolify honours a Dockerfile HEALTHCHECK. Each request leaves
# one JSON line (method, path, status, duration_ms) in the container log, nothing about the caller.

# Bookworm, to match the distroless debian12 runtime's glibc.
FROM rust:1.90-slim-bookworm AS build
ARG BIN=standards
WORKDIR /app

# Dependencies first, from the manifests and the one lockfile alone, so an edit rebuilds in
# seconds. Every build is --locked: without it the image resolves the newest crates on each
# deploy, and a release could stop building the day a dependency needs a newer Rust.
COPY Cargo.toml Cargo.lock ./
COPY workers/standards-api/Cargo.toml workers/standards-api/
COPY site/Cargo.toml site/
RUN mkdir -p src site/src workers/standards-api/src \
 && : > site/src/lib.rs \
 && echo 'fn main() {}' > site/build.rs \
 && echo 'fn main() {}' > src/main.rs \
 && echo 'fn main() {}' > workers/standards-api/src/main.rs \
 && : > workers/standards-api/src/lib.rs \
 && echo 'fn main() {}' > workers/standards-api/build.rs \
 && case "$BIN" in standards) PKG=bundu-standards-api ;; vocab) PKG=bundu-vocab ;; *) echo "BIN must be standards or vocab" >&2; exit 1 ;; esac \
 && cargo build --release --locked -p "$PKG" \
 && rm -rf src site/src site/build.rs workers/standards-api/src workers/standards-api/build.rs

COPY src ./src
COPY site ./site
COPY workers/standards-api ./workers/standards-api
COPY register ./register
COPY standards ./standards
# Touch, so cargo notices the real sources replaced the stubs.
RUN touch src/main.rs site/build.rs site/src/lib.rs workers/standards-api/build.rs workers/standards-api/src/main.rs workers/standards-api/src/lib.rs \
 && case "$BIN" in standards) PKG=bundu-standards-api; EXE=bundu-standards ;; vocab) PKG=bundu-vocab; EXE=bundu-vocab ;; esac \
 && cargo build --release --locked -p "$PKG" --bin "$EXE" \
 && cp "target/release/$EXE" /server

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=build /server /server
ENV LISTEN_ADDR=0.0.0.0:8080
EXPOSE 8080
USER nonroot:nonroot
HEALTHCHECK --interval=30s --timeout=5s --start-period=10s --retries=3 CMD ["/server", "healthcheck"]
ENTRYPOINT ["/server"]
