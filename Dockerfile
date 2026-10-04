# vocab.bundu.org — a namespace server has to stay up for years, so the
# deployed artefact is deliberately boring: one static-ish binary on a
# distroless base, no runtime dependencies to rot.

# Bookworm, to match the distroless debian12 runtime's glibc.
FROM rust:1.90-slim-bookworm AS build
WORKDIR /app

# Cache dependencies separately from source so term edits rebuild in seconds.
# Cargo.lock comes too, and every build is --locked: without it the image
# resolves the newest crates on each deploy, which is how a dependency that
# needs a newer Rust broke the 0.1.1 deploy.
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
 && cargo build --release --locked \
 && rm -rf src

COPY src ./src
# Touch so cargo notices the real main.rs replaced the stub.
RUN touch src/main.rs && cargo build --release --locked

FROM gcr.io/distroless/cc-debian12
COPY --from=build /app/target/release/bundu-vocab /bundu-vocab
ENV LISTEN_ADDR=0.0.0.0:8080
EXPOSE 8080
ENTRYPOINT ["/bundu-vocab"]
