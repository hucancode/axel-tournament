.PHONY: test-db-up test-db-down test-mail-server-up test-mail-server-down

# Container runtime (docker or podman)
DATABASE_PORT ?= 8000
DATABASE_URL ?= ws://localhost:$(DATABASE_PORT)
SMTP_PORT ?= 1025
SMTP_WEB_PORT ?= 8025

# Start test database (in-memory SurrealDB)
test-db-up:
	@echo "Starting test SurrealDB instance on port $(DATABASE_PORT)..."
	@docker run -d --rm \
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
	@docker stop surrealdb-test

# Start test mail server (Mailpit)
test-mail-server-up:
	@echo "Starting Mailpit instance on SMTP port $(SMTP_PORT) and web UI port $(SMTP_WEB_PORT)..."
	@docker run -d --rm \
		--name mailpit-test \
		-p $(SMTP_PORT):1025 \
		-p $(SMTP_WEB_PORT):8025 \
		axllent/mailpit:latest
	@echo "Waiting for mail server to be ready..."
	@sleep 2
	@echo "Test mail server ready:"
	@echo "  SMTP: localhost:$(SMTP_PORT)"
	@echo "  Web UI: http://localhost:$(SMTP_WEB_PORT)"

# Stop test mail server
test-mail-server-down:
	@echo "Stopping Mailpit instance..."
	@docker stop mailpit-test
