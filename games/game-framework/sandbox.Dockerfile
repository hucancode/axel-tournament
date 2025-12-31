# Minimal runtime image for executing compiled user binaries
# Compilation happens in separate standard compiler images
FROM alpine:3.23

# Install only runtime dependencies needed for executing binaries
RUN apk add --no-cache \
    libgcc \
    libstdc++ \
    ca-certificates

# Create non-root user for running code
RUN adduser -D -u 1000 sandbox

WORKDIR /sandbox
USER sandbox

# Compiled binaries will be copied in at runtime
CMD ["/bin/sh"]
