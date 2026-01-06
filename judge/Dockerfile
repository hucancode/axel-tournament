FROM rust:slim-trixie

WORKDIR /app

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev golang-go gcc g++ binutils libc6-dev && \
    rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock* ./
COPY src ./src

RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    --mount=type=cache,target=/app/target \
     cargo build --release

EXPOSE 8080

CMD ["./target/release/judge"]
