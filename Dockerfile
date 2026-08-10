# vocab.bundu.org — a namespace server has to stay up for years, so the
# deployed artefact is deliberately boring: one static-ish binary on a
# distroless base, no runtime dependencies to rot.

FROM rust:1.83-slim AS build
WORKDIR /app

# Cache dependencies separately from source so term edits rebuild in seconds.
COPY Cargo.toml ./
RUN mkdir src && echo 'fn main() {}' > src/main.rs \
 && cargo build --release \
 && rm -rf src

COPY src ./src
# Touch so cargo notices the real main.rs replaced the stub.
RUN touch src/main.rs && cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=build /app/target/release/bundu-vocab /bundu-vocab
ENV LISTEN_ADDR=0.0.0.0:8080
EXPOSE 8080
ENTRYPOINT ["/bundu-vocab"]
