# Universal Game Runner Dockerfile
# Supports Rust, Go, C, and Python game servers and player submissions

FROM debian:bookworm-slim

ENV DEBIAN_FRONTEND=noninteractive \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH="/usr/local/cargo/bin:${PATH}"

RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        curl \
        golang \
        python3 && \
    rm -rf /var/lib/apt/lists/* \
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable && \
    rm -rf /usr/local/cargo/registry /usr/local/cargo/git /root/.cache &&\
    rm -rf /usr/share/{doc,man,locale}

WORKDIR /game
COPY sandbox-entrypoint.sh /game/entrypoint.sh
RUN chmod +x /game/entrypoint.sh
ENTRYPOINT ["/bin/bash", "/game/entrypoint.sh"]
