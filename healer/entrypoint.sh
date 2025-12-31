#!/bin/bash
set -e

# Wait for dockerd to be ready
echo "Waiting for Docker daemon..."
while ! docker version >/dev/null 2>&1; do
    sleep 2
done
echo "Docker daemon is ready"

# Load compiler images
echo "Loading Rust compiler image..."
docker load < rust-compiler.tar

echo "Loading Go compiler image..."
docker load < go-compiler.tar

echo "Loading C compiler image..."
docker load < c-compiler.tar

echo "All compiler images loaded successfully"

# Start healer
echo "Starting healer service..."
exec ./healer
