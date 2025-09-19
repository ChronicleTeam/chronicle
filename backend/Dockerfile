FROM docker.io/lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY migrations migrations
COPY src src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
COPY migrations migrations
COPY src src
RUN cargo build --release

FROM docker.io/debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    libssl3 \
    ca-certificates \
 && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/chronicle /usr/local/bin
ENTRYPOINT ["/usr/local/bin/chronicle"]