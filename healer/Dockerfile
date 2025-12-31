FROM rust:1.92-slim AS builder

WORKDIR /app

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
    cargo build --release && \
    cp /app/target/release/axel-healer axel-healer

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 docker.io && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/axel-healer healer
COPY ../games/game-framework/target/rust-compiler.tar rust-compiler.tar
COPY ../games/game-framework/target/go-compiler.tar go-compiler.tar
COPY ../games/game-framework/target/c-compiler.tar c-compiler.tar
COPY entrypoint.sh entrypoint.sh

RUN chmod +x entrypoint.sh

ENTRYPOINT ["./entrypoint.sh"]
