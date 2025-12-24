#!/bin/bash

# Wait for dockerd to be ready
while ! docker version >/dev/null 2>&1; do
    echo "Waiting for Docker daemon..."
    sleep 2
done

# Load sandbox image
echo "Loading sandbox image..."
docker load < /app/sandbox.tar

# Start judge
exec /app/judge
