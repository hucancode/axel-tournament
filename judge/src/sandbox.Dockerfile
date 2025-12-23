# Universal Game Runner Dockerfile
# Supports Rust, Go, C, and Python game servers and player submissions

FROM debian:bookworm-slim

WORKDIR /workspace

ENV DEBIAN_FRONTEND=noninteractive \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo

RUN apt-get update && \
    apt-get install -y --no-install-recommends gcc ca-certificates curl golang && \
    rm -rf /var/lib/apt/lists/* && \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable && \
    rm -rf /usr/local/cargo/{registry,git} /root/.cache &&\
    rm -rf /usr/share/{doc,man,locale} && \
    rm -rf /usr/share/go-*/src

COPY sandbox-entrypoint.sh entrypoint.sh
RUN chmod +x entrypoint.sh
ENTRYPOINT ["/bin/bash", "entrypoint.sh"]
