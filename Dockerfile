ARG RUST_VERSION=1

FROM rust:${RUST_VERSION}-bookworm AS builder

# Dependencies for building static musl binaries; rustls 0.23 may pull in aws-lc-rs.
RUN apt-get update \
 && apt-get install -y --no-install-recommends \
      musl-tools clang cmake perl pkg-config ca-certificates \
 && rm -rf /var/lib/apt/lists/*

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /workspace

# Pre-copy manifests to leverage Docker layer caching.
COPY Cargo.toml Cargo.lock ./
COPY genkey/Cargo.toml genkey/Cargo.toml
COPY synchronous_server/Cargo.toml synchronous_server/Cargo.toml
COPY synchronous_client/Cargo.toml synchronous_client/Cargo.toml

# Cargo needs workspace members to have at least one target; create dummy sources
# for the dependency-fetch layer, then overwrite with the real sources later.
RUN mkdir -p genkey/src synchronous_server/src synchronous_client/src \
 && printf 'fn main() {}\n' > genkey/src/main.rs \
 && printf 'fn main() {}\n' > synchronous_server/src/main.rs \
 && printf 'fn main() {}\n' > synchronous_client/src/main.rs

RUN cargo fetch

# Now copy real sources.
COPY genkey/src genkey/src
COPY synchronous_server/src synchronous_server/src
COPY synchronous_client/src synchronous_client/src

RUN cargo build --release --locked --target x86_64-unknown-linux-musl \
    -p synchronous_server -p synchronous_client

FROM scratch AS server
WORKDIR /app
COPY --from=builder /workspace/target/x86_64-unknown-linux-musl/release/synchronous_server /synchronous_server
USER 65532:65532
ENTRYPOINT ["/synchronous_server"]


FROM scratch AS client
WORKDIR /app
COPY --from=builder /workspace/target/x86_64-unknown-linux-musl/release/synchronous_client /synchronous_client
USER 65532:65532
ENTRYPOINT ["/synchronous_client"]
