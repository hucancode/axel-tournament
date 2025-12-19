.PHONY: test-db-up test-db-down aws-up aws-down k8s-up k8s-down

# Container runtime (docker or podman)
CONTAINER_RUNTIME := docker

CLUSTER_NAME ?= axel-eks

ECR_FRONTEND_REPO ?= $(shell terraform -chdir=infra output -raw ecr_frontend_repository 2>/dev/null)
ECR_BACKEND_REPO ?= $(shell terraform -chdir=infra output -raw ecr_backend_repository 2>/dev/null)
ECR_JUDGE_REPO ?= $(shell terraform -chdir=infra output -raw ecr_judge_repository 2>/dev/null)

FRONTEND_IMAGE ?= $(if $(ECR_FRONTEND_REPO),$(ECR_FRONTEND_REPO):latest,)
BACKEND_IMAGE ?= $(if $(ECR_BACKEND_REPO),$(ECR_BACKEND_REPO):latest,)
JUDGE_IMAGE ?= $(if $(ECR_JUDGE_REPO),$(ECR_JUDGE_REPO):latest,)
ENV_SUBST_VARS := '$${FRONTEND_IMAGE} $${BACKEND_IMAGE} $${JUDGE_IMAGE}'

export FRONTEND_IMAGE
export BACKEND_IMAGE
export JUDGE_IMAGE
# Database settings
DATABASE_PORT ?= 8001
DATABASE_URL ?= ws://localhost:$(DATABASE_PORT)

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

aws-up:
	@terraform -chdir=infra init
	@terraform -chdir=infra apply
	@aws eks update-kubeconfig --name $(CLUSTER_NAME)
	@helm repo add ingress-nginx https://kubernetes.github.io/ingress-nginx
	@helm repo add metrics-server https://kubernetes-sigs.github.io/metrics-server/
	@helm repo update
	@helm upgrade --install ingress-nginx ingress-nginx/ingress-nginx \
		--namespace ingress \
		--create-namespace \
		--set controller.service.type=LoadBalancer
	@helm upgrade --install metrics-server metrics-server/metrics-server \
		--namespace kube-system \
		--set args={--kubelet-insecure-tls}

k8s-up:
	@test -n "$(FRONTEND_IMAGE)" || (echo "FRONTEND_IMAGE is required" && exit 1)
	@test -n "$(BACKEND_IMAGE)" || (echo "BACKEND_IMAGE is required" && exit 1)
	@test -n "$(JUDGE_IMAGE)" || (echo "JUDGE_IMAGE is required" && exit 1)
	@kubectl apply -f k8s/namespaces.yaml
	@kubectl apply -f k8s/addons/gp3-storageclass.yaml
	@kubectl apply -f k8s/apps/surrealdb.yaml
	@envsubst $(ENV_SUBST_VARS) < k8s/apps/backend.yaml | kubectl apply -f -
	@envsubst $(ENV_SUBST_VARS) < k8s/apps/judge.yaml | kubectl apply -f -
	@envsubst $(ENV_SUBST_VARS) < k8s/apps/frontend.yaml | kubectl apply -f -

k8s-down:
	@envsubst $(ENV_SUBST_VARS) < k8s/apps/frontend.yaml | kubectl delete -f - --ignore-not-found
	@envsubst $(ENV_SUBST_VARS) < k8s/apps/judge.yaml | kubectl delete -f - --ignore-not-found
	@envsubst $(ENV_SUBST_VARS) < k8s/apps/backend.yaml | kubectl delete -f - --ignore-not-found
	@kubectl delete -f k8s/apps/surrealdb.yaml --ignore-not-found
	@kubectl delete -f k8s/addons/gp3-storageclass.yaml --ignore-not-found
	@kubectl delete -f k8s/namespaces.yaml --ignore-not-found

aws-down: k8s-down
	@helm uninstall ingress-nginx --namespace ingress || true
	@helm uninstall metrics-server --namespace kube-system || true
	@terraform -chdir=infra destroy
