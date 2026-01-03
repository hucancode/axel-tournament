#!/bin/bash
set -e

# Wait for dockerd to be ready
echo "Waiting for Docker daemon..."
while ! docker version >/dev/null 2>&1; do
    sleep 2
done
echo "Docker daemon is ready"

# Load compiler images
echo "Loading sandbox image..."
docker load < sandbox.tar

# Start healer
echo "Starting judge service..."
exec ./judge

