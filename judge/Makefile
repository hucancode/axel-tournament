.PHONY: sandbox-image

# Container runtime (docker or podman)
CONTAINER_RUNTIME := docker
SANDBOX_IMAGE ?= axel-sandbox

# sandbox image used to run matches
sandbox-image:
	@$(CONTAINER_RUNTIME) build -f src/sandbox.Dockerfile -t $(SANDBOX_IMAGE) src
