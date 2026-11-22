# One build for both sites. Coolify runs two apps from this repository and this file; they
# differ only in the BIN build argument and the domain:
#
#   BIN=standards  (the default)  standards.bundu.org  workers/standards-api
#   BIN=vocab                     vocab.bundu.org      the root crate (src/)
#
# Build from the repository root. The standards build reads register/ and standards/ as well as
# its crate (build.rs), so they are in the context (.dockerignore keeps the rest out). Nothing
# is read at run time: everything the server serves is inside the binary.
#
# Both listen on LISTEN_ADDR (default 0.0.0.0:8080), answer GET /healthz with {"status":"ok"},
# and run as the distroless non-root user. There is no HEALTHCHECK on purpose: the runtime image
# has no shell and no curl, so a check inside the container cannot work. Check /healthz from
# outside it (Coolify's health check path, a monitor).

# Bookworm, to match the distroless debian12 runtime's glibc.
FROM rust:1.90-slim-bookworm AS build
ARG BIN=standards
WORKDIR /app

# Dependencies first, from the manifests and the one lockfile alone, so an edit rebuilds in
# seconds. Every build is --locked: without it the image resolves the newest crates on each
# deploy, and a release could stop building the day a dependency needs a newer Rust.
COPY Cargo.toml Cargo.lock ./
COPY workers/standards-api/Cargo.toml workers/standards-api/
RUN mkdir -p src workers/standards-api/src \
 && echo 'fn main() {}' > src/main.rs \
 && echo 'fn main() {}' > workers/standards-api/src/main.rs \
 && : > workers/standards-api/src/lib.rs \
 && echo 'fn main() {}' > workers/standards-api/build.rs \
 && case "$BIN" in standards) PKG=bundu-standards-api ;; vocab) PKG=bundu-vocab ;; *) echo "BIN must be standards or vocab" >&2; exit 1 ;; esac \
 && cargo build --release --locked -p "$PKG" \
 && rm -rf src workers/standards-api/src workers/standards-api/build.rs

COPY src ./src
COPY workers/standards-api ./workers/standards-api
COPY register ./register
COPY standards ./standards
# Touch, so cargo notices the real sources replaced the stubs.
RUN touch src/main.rs workers/standards-api/build.rs workers/standards-api/src/main.rs workers/standards-api/src/lib.rs \
 && case "$BIN" in standards) PKG=bundu-standards-api; EXE=bundu-standards ;; vocab) PKG=bundu-vocab; EXE=bundu-vocab ;; esac \
 && cargo build --release --locked -p "$PKG" --bin "$EXE" \
 && cp "target/release/$EXE" /server

FROM gcr.io/distroless/cc-debian12:nonroot
COPY --from=build /server /server
ENV LISTEN_ADDR=0.0.0.0:8080
EXPOSE 8080
USER nonroot:nonroot
ENTRYPOINT ["/server"]
