.PHONY: test-db test-db-stop judge-server judge-server-stop test clean

# Container runtime (docker or podman)
CONTAINER_RUNTIME := $(shell which podman 2>/dev/null || which docker 2>/dev/null || echo docker)
JUDGE_IMAGE ?= axel-judge:latest
JUDGE_CONTAINER ?= judge-server

# Database settings
DATABASE_PORT ?= 8001
DATABASE_URL ?= ws://localhost:$(DATABASE_PORT)

# Start test database (in-memory SurrealDB on configurable port)
test-db:
	@echo "Starting test SurrealDB instance on port $(DATABASE_PORT)..."
	@$(CONTAINER_RUNTIME) run -d --rm \
		--name surrealdb-test \
		-p $(DATABASE_PORT):8000 \
		surrealdb/surrealdb:latest \
		start --user root --pass root memory
	@echo "Waiting for database to be ready..."
	@sleep 2
	@echo "Test database ready at ws://localhost:$(DATABASE_PORT)"

# Stop test database
test-db-stop:
	@echo "Stopping test SurrealDB instance..."
	@$(CONTAINER_RUNTIME) stop surrealdb-test 2>/dev/null || true
	@echo "Test database stopped"

# Start judge server (database-only dependency)
judge-server:
	@echo "Building judge image ($(JUDGE_IMAGE))..."
	@$(CONTAINER_RUNTIME) build -t $(JUDGE_IMAGE) ./judge
	@echo "Starting judge server container ($(JUDGE_CONTAINER))..."
	@$(CONTAINER_RUNTIME) run -d --rm \
		--name $(JUDGE_CONTAINER) \
		-v $(CURDIR)/judge/Dockerfile.universal:/app/Dockerfile.universal:ro \
		-e DATABASE_URL=$${DATABASE_URL} \
		$(JUDGE_IMAGE)

# Stop judge server container
judge-server-stop:
	@echo "Stopping judge server container..."
	@$(CONTAINER_RUNTIME) stop $(JUDGE_CONTAINER) 2>/dev/null || true
	@echo "Judge server stopped"

# Clean up everything
clean: test-db-stop judge-server-stop
