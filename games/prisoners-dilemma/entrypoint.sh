#!/bin/bash
set -e

# Wait for dockerd to be ready
echo "Waiting for Docker daemon..."
while ! docker version >/dev/null 2>&1; do
    sleep 2
done
echo "Docker daemon is ready"

# Load sandbox runtime image
echo "Loading sandbox runtime image..."
docker load < sandbox.tar

echo "Sandbox image loaded successfully"

# Start game server
echo "Starting Prisoner's Dilemma game server..."
exec ./server
