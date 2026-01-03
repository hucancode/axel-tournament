FROM rust:slim-trixie AS builder

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
    cp /app/target/release/judge judge

FROM debian:trixie-slim

RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 docker.io && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/judge judge
COPY sandbox.tar sandbox.tar
COPY entrypoint.sh entrypoint.sh
RUN chmod +x entrypoint.sh

EXPOSE 8080

ENTRYPOINT ["./entrypoint.sh"]
