.PHONY: test-db test-db-stop test clean help

# Container runtime (docker or podman)
CONTAINER_RUNTIME := $(shell which podman 2>/dev/null || which docker 2>/dev/null || echo docker)

# Start test database (in-memory SurrealDB on port 8001)
test-db:
	@echo "Starting test SurrealDB instance on port 8001..."
	@$(CONTAINER_RUNTIME) run -d --rm \
		--name surrealdb-test \
		-p 8001:8000 \
		surrealdb/surrealdb:latest \
		start --user root --pass root memory
	@echo "Waiting for database to be ready..."
	@sleep 2
	@echo "Test database ready at ws://localhost:8001"

# Stop test database
test-db-stop:
	@echo "Stopping test SurrealDB instance..."
	@$(CONTAINER_RUNTIME) stop surrealdb-test 2>/dev/null || true
	@echo "Test database stopped"

# Run tests with test database
test: test-db
	@echo "Running tests..."
	@cargo test || ($(MAKE) test-db-stop && exit 1)
	@$(MAKE) test-db-stop

# Clean up everything
clean: test-db-stop
	@echo "Cleaning build artifacts..."
	@cargo clean

# Show help
help:
	@echo "Available targets:"
	@echo "  test-db       - Start in-memory SurrealDB on port 8001"
	@echo "  test-db-stop  - Stop test SurrealDB instance"
	@echo "  test          - Run tests with test database (auto start/stop)"
	@echo "  clean         - Stop database and clean build artifacts"
	@echo "  help          - Show this help message"
