.PHONY: aws-up aws-down k8s-up k8s-down sandbox-image

CLUSTER_NAME ?= axel-eks
CONTAINER_RUNTIME ?= docker

ECR_FRONTEND_REPO ?= $(shell terraform output -raw ecr_frontend_repository 2>/dev/null)
ECR_BACKEND_REPO ?= $(shell terraform output -raw ecr_backend_repository 2>/dev/null)
ECR_JUDGE_REPO ?= $(shell terraform output -raw ecr_judge_repository 2>/dev/null)
ECR_SANDBOX_REPO ?= $(shell terraform output -raw ecr_sandbox_repository 2>/dev/null)
ECR_REGISTRY ?= $(word 1, $(subst /, ,$(ECR_SANDBOX_REPO)))

FRONTEND_IMAGE ?= $(if $(ECR_FRONTEND_REPO),$(ECR_FRONTEND_REPO):latest,)
BACKEND_IMAGE ?= $(if $(ECR_BACKEND_REPO),$(ECR_BACKEND_REPO):latest,)
JUDGE_IMAGE ?= $(if $(ECR_JUDGE_REPO),$(ECR_JUDGE_REPO):latest,)
SANDBOX_IMAGE ?= $(if $(ECR_SANDBOX_REPO),$(ECR_SANDBOX_REPO):latest,)
ENV_SUBST_VARS := '$${FRONTEND_IMAGE} $${BACKEND_IMAGE} $${JUDGE_IMAGE} $${SANDBOX_IMAGE}'

export FRONTEND_IMAGE
export BACKEND_IMAGE
export JUDGE_IMAGE
export SANDBOX_IMAGE

aws-up:
	@terraform init
	@terraform apply
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

k8s-up: sandbox-image
	@test -n "$(FRONTEND_IMAGE)" || (echo "FRONTEND_IMAGE is required" && exit 1)
	@test -n "$(BACKEND_IMAGE)" || (echo "BACKEND_IMAGE is required" && exit 1)
	@test -n "$(JUDGE_IMAGE)" || (echo "JUDGE_IMAGE is required" && exit 1)
	@test -n "$(SANDBOX_IMAGE)" || (echo "SANDBOX_IMAGE is required" && exit 1)
	@kubectl apply -f k8s/namespaces.yaml
	@kubectl apply -f k8s/addons/gp3-storageclass.yaml
	@kubectl apply -f k8s/apps/surrealdb.yaml
	@envsubst $(ENV_SUBST_VARS) < k8s/apps/backend.yaml | kubectl apply -f -
	@envsubst $(ENV_SUBST_VARS) < k8s/apps/judge.yaml | kubectl apply -f -
	@envsubst $(ENV_SUBST_VARS) < k8s/apps/frontend.yaml | kubectl apply -f -

sandbox-image:
	@test -n "$(SANDBOX_IMAGE)" || (echo "SANDBOX_IMAGE is required" && exit 1)
	@test -n "$(ECR_SANDBOX_REPO)" || (echo "ECR_SANDBOX_REPO is required" && exit 1)
	@echo "Logging in to ECR $(ECR_REGISTRY)..."
	@aws ecr get-login-password | $(CONTAINER_RUNTIME) login --username AWS --password-stdin $(ECR_REGISTRY)
	@$(CONTAINER_RUNTIME) build -f ../judge/src/sandbox.Dockerfile -t $(SANDBOX_IMAGE) ../judge/src
	@$(CONTAINER_RUNTIME) push $(SANDBOX_IMAGE)

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
	@terraform destroy
