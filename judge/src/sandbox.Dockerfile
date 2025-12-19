# Universal Game Runner Dockerfile
# Supports Rust, Go, C, and Python game servers and player submissions

FROM debian:bookworm-slim

# Prevent interactive prompts during installation and set toolchain paths
ENV DEBIAN_FRONTEND=noninteractive \
    RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH="/usr/local/cargo/bin:${PATH}"

# Install minimal build/runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        build-essential \
        ca-certificates \
        curl \
        git \
        golang \
        python3 \
        python3-pip \
    && rm -rf /var/lib/apt/lists/*

# Install Rust with minimal profile and clean caches
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
        | sh -s -- -y --profile minimal --default-toolchain stable && \
    rm -rf /usr/local/cargo/registry /usr/local/cargo/git /root/.cache

WORKDIR /game

# Add universal entrypoint script
COPY sandbox-entrypoint.sh /game/entrypoint.sh
RUN chmod +x /game/entrypoint.sh

# Set entrypoint
ENTRYPOINT ["/bin/bash", "/game/entrypoint.sh"]
