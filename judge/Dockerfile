FROM rust:1.92-slim AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock* ./
COPY src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cp /app/target/release/axel-judge /app/axel-judge

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 docker.io && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/axel-judge judge
COPY sandbox.tar sandbox.tar
COPY entrypoint.sh entrypoint.sh

RUN chmod +x entrypoint.sh

CMD ["entrypoint.sh"]
