# Stage 1: Build the application
FROM rust:1.92-bookworm AS builder

WORKDIR /app

# Copy the game framework first
COPY ../game-framework /app/game-framework

# Copy the game server
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build the application
RUN cargo build --release

# Stage 2: Runtime
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y ca-certificates libssl3 docker.io && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/tic-tac-toe server
COPY --from=builder /app/game-framework/target/sandbox.tar sandbox.tar
COPY entrypoint.sh entrypoint.sh

RUN chmod +x entrypoint.sh

EXPOSE 8084

ENTRYPOINT ["./entrypoint.sh"]
