.PHONY: test-db-up test-db-down sandbox-image

# Container runtime (docker or podman)
CONTAINER_RUNTIME := docker
DATABASE_PORT ?= 8001
DATABASE_URL ?= ws://localhost:$(DATABASE_PORT)
SANDBOX_IMAGE ?= axel-sandbox

# Start test database (in-memory SurrealDB)
test-db-up:
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
test-db-down:
	@echo "Stopping test SurrealDB instance..."
	@$(CONTAINER_RUNTIME) stop surrealdb-test

# sandbox image used to run matches
sandbox-image:
	@$(CONTAINER_RUNTIME) build -f ../judge/src/sandbox.Dockerfile -t $(SANDBOX_IMAGE) ../judge/src
