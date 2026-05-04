# ───── builder ───────────────────────────────────────────────────────────
FROM rust:1.83-slim-bookworm AS builder

WORKDIR /build

# Install build deps for openssl-sys (only if needed; rustls builds slimmer).
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for build caching.
COPY Cargo.toml ./
COPY crates/bundu-identity-core/Cargo.toml crates/bundu-identity-core/
COPY crates/bundu-identity-server/Cargo.toml crates/bundu-identity-server/
COPY crates/bundu-jwt/Cargo.toml crates/bundu-jwt/
COPY crates/bundu-workos/Cargo.toml crates/bundu-workos/
COPY crates/bundu-supabase/Cargo.toml crates/bundu-supabase/
COPY crates/bundu-service-bus/Cargo.toml crates/bundu-service-bus/

# Stub source files so cargo can resolve and warm the dependency cache.
RUN mkdir -p \
    crates/bundu-identity-core/src \
    crates/bundu-identity-server/src \
    crates/bundu-jwt/src \
    crates/bundu-workos/src \
    crates/bundu-supabase/src \
    crates/bundu-service-bus/src \
    && echo "" > crates/bundu-identity-core/src/lib.rs \
    && echo "" > crates/bundu-jwt/src/lib.rs \
    && echo "" > crates/bundu-workos/src/lib.rs \
    && echo "" > crates/bundu-supabase/src/lib.rs \
    && echo "" > crates/bundu-service-bus/src/lib.rs \
    && echo "fn main() {}" > crates/bundu-identity-server/src/main.rs \
    && echo "" > crates/bundu-identity-server/src/lib.rs

RUN cargo build --release -p bundu-identity-server || true

# Now copy the actual source.
COPY crates/ crates/

RUN touch crates/*/src/lib.rs crates/*/src/main.rs 2>/dev/null || true
RUN cargo build --release -p bundu-identity-server

# ───── runtime ───────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --system --uid 10001 --create-home bundu

COPY --from=builder /build/target/release/bundu-identity-server /usr/local/bin/bundu-identity-server

USER bundu
EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/bundu-identity-server"]
