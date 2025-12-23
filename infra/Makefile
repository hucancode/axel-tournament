.PHONY: aws-up aws-down k8s-up k8s-down sandbox-image
.ONESHELL:

# Independent variables
CLUSTER_NAME ?= axel-eks
CONTAINER_RUNTIME ?= docker
SMTP_PORT := 587
DEFAULT_EMAIL_FROM := noreply@axel-tournament.com
DEFAULT_PUBLIC_API_URL := http://backend.backend.svc.cluster.local:8080

# Environment variable templates for envsubst
ENV_SUBST_VARS := '$${FRONTEND_IMAGE} $${BACKEND_IMAGE} $${JUDGE_IMAGE} $${SANDBOX_IMAGE} $${HEALER_IMAGE} $${SMTP_HOST} $${SMTP_PORT} $${SMTP_USERNAME} $${SMTP_PASSWORD} $${EMAIL_FROM} $${PUBLIC_API_URL}'
LB_CONTROLLER_ENV_VARS := '$${AWS_LB_CONTROLLER_ROLE_ARN}'

aws-up:
	@terraform init
	@terraform apply -auto-approve
	@aws eks update-kubeconfig --name $(CLUSTER_NAME)
	@helm repo add eks https://aws.github.io/eks-charts
	@helm repo add metrics-server https://kubernetes-sigs.github.io/metrics-server/
	@helm repo update
	@AWS_LB_CONTROLLER_ROLE_ARN=$$(terraform output -raw aws_load_balancer_controller_role_arn 2>/dev/null)
	@test -n "$$AWS_LB_CONTROLLER_ROLE_ARN" || (echo "AWS_LB_CONTROLLER_ROLE_ARN is required" && exit 1)
	@export AWS_LB_CONTROLLER_ROLE_ARN
	@envsubst $(LB_CONTROLLER_ENV_VARS) < k8s/addons/aws-load-balancer-controller-service-account.yaml | kubectl apply -f -
	@AWS_REGION=$$(terraform output -raw aws_region 2>/dev/null)
	@VPC_ID=$$(terraform output -raw vpc_id 2>/dev/null)
	@test -n "$$VPC_ID" || (echo "VPC_ID is required" && exit 1)
	@helm upgrade --install aws-load-balancer-controller eks/aws-load-balancer-controller \
		--namespace kube-system \
		--set clusterName=$(CLUSTER_NAME) \
		--set serviceAccount.create=false \
		--set serviceAccount.name=aws-load-balancer-controller \
		--set region=$$AWS_REGION \
		--set vpcId=$$VPC_ID
	@echo "Waiting for AWS Load Balancer Controller to be ready..."
	@kubectl wait --namespace kube-system \
		--for=condition=ready pod \
		--selector=app.kubernetes.io/name=aws-load-balancer-controller \
		--timeout=90s
	@helm upgrade --install metrics-server metrics-server/metrics-server \
		--namespace kube-system \
		--set args={--kubelet-insecure-tls}

k8s-up: sandbox-image
	@ECR_WEB_REPO=$$(terraform output -raw ecr_web_repository 2>/dev/null)
	@ECR_API_REPO=$$(terraform output -raw ecr_api_repository 2>/dev/null)
	@ECR_JUDGE_REPO=$$(terraform output -raw ecr_judge_repository 2>/dev/null)
	@ECR_SANDBOX_REPO=$$(terraform output -raw ecr_sandbox_repository 2>/dev/null)
	@ECR_HEALER_REPO=$$(terraform output -raw ecr_healer_repository 2>/dev/null)
	@SES_SMTP_HOST=$$(terraform output -raw ses_smtp_host 2>/dev/null)
	@SES_SMTP_USERNAME=$$(terraform output -raw ses_smtp_username 2>/dev/null)
	@SES_SMTP_PASSWORD=$$(terraform output -raw ses_smtp_password 2>/dev/null)
	@SES_EMAIL_IDENTITY=$$(terraform output -raw ses_email_identity 2>/dev/null)
	@SES_DOMAIN=$$(terraform output -raw ses_domain 2>/dev/null)
	@export FRONTEND_IMAGE="$$ECR_WEB_REPO:latest"
	@export BACKEND_IMAGE="$$ECR_API_REPO:latest"
	@export JUDGE_IMAGE="$$ECR_JUDGE_REPO:latest"
	@export SANDBOX_IMAGE="$$ECR_SANDBOX_REPO:latest"
	@export HEALER_IMAGE="$$ECR_HEALER_REPO:latest"
	@export SMTP_HOST="$$SES_SMTP_HOST"
	@export SMTP_PORT=$(SMTP_PORT)
	@export SMTP_USERNAME="$$SES_SMTP_USERNAME"
	@export SMTP_PASSWORD="$$SES_SMTP_PASSWORD"
	@export EMAIL_FROM=$$(if [ -n "$$SES_EMAIL_IDENTITY" ]; then echo "$$SES_EMAIL_IDENTITY"; elif [ -n "$$SES_DOMAIN" ]; then echo "noreply@$$SES_DOMAIN"; else echo "$(DEFAULT_EMAIL_FROM)"; fi)
	@export PUBLIC_API_URL="$(DEFAULT_PUBLIC_API_URL)"
	@test -n "$$FRONTEND_IMAGE" || (echo "FRONTEND_IMAGE is required" && exit 1)
	@test -n "$$BACKEND_IMAGE" || (echo "BACKEND_IMAGE is required" && exit 1)
	@test -n "$$JUDGE_IMAGE" || (echo "JUDGE_IMAGE is required" && exit 1)
	@test -n "$$SANDBOX_IMAGE" || (echo "SANDBOX_IMAGE is required" && exit 1)
	@test -n "$$HEALER_IMAGE" || (echo "HEALER_IMAGE is required" && exit 1)
	@kubectl apply -f k8s/namespaces.yaml
	@kubectl apply -f k8s/addons/gp3-storageclass.yaml
	@kubectl apply -f k8s/apps/surrealdb.yaml
	@envsubst $(ENV_SUBST_VARS) < k8s/apps/backend.yaml | kubectl apply -f -
	@envsubst $(ENV_SUBST_VARS) < k8s/apps/judge.yaml | kubectl apply -f -
	@envsubst $(ENV_SUBST_VARS) < k8s/apps/healer.yaml | kubectl apply -f -
	@backend_host=""
	@i=0
	@while [ -z "$$backend_host" ] && [ $$i -lt 30 ]; do \
		i=$$((i+1)); \
		backend_host=$$(kubectl -n backend get ingress backend -o jsonpath='{.status.loadBalancer.ingress[0].hostname}' 2>/dev/null); \
		if [ -n "$$backend_host" ]; then break; fi; \
		echo "Waiting for backend ingress hostname ($$i/30)..."; \
		sleep 10; \
	done
	@if [ -z "$$backend_host" ]; then \
		echo "Backend ingress hostname not available; set PUBLIC_API_URL and re-run k8s-up."; \
		exit 1; \
	fi
	@PUBLIC_API_URL="http://$$backend_host" envsubst $(ENV_SUBST_VARS) < k8s/apps/frontend.yaml | kubectl apply -f -

sandbox-image:
	@ECR_SANDBOX_REPO=$$(terraform output -raw ecr_sandbox_repository 2>/dev/null)
	@test -n "$$ECR_SANDBOX_REPO" || (echo "ECR_SANDBOX_REPO is required" && exit 1)
	@SANDBOX_IMAGE="$$ECR_SANDBOX_REPO:latest"
	@ECR_REGISTRY=$$(echo "$$ECR_SANDBOX_REPO" | cut -d/ -f1)
	@echo "Logging in to ECR $$ECR_REGISTRY..."
	@aws ecr get-login-password | $(CONTAINER_RUNTIME) login --username AWS --password-stdin $$ECR_REGISTRY
	@$(CONTAINER_RUNTIME) build -f ../judge/src/sandbox.Dockerfile -t $$SANDBOX_IMAGE ../judge/src
	@$(CONTAINER_RUNTIME) push $$SANDBOX_IMAGE

k8s-down:
	@ECR_WEB_REPO=$$(terraform output -raw ecr_web_repository 2>/dev/null)
	@ECR_API_REPO=$$(terraform output -raw ecr_api_repository 2>/dev/null)
	@ECR_JUDGE_REPO=$$(terraform output -raw ecr_judge_repository 2>/dev/null)
	@ECR_SANDBOX_REPO=$$(terraform output -raw ecr_sandbox_repository 2>/dev/null)
	@ECR_HEALER_REPO=$$(terraform output -raw ecr_healer_repository 2>/dev/null)
	@SES_SMTP_HOST=$$(terraform output -raw ses_smtp_host 2>/dev/null)
	@SES_SMTP_USERNAME=$$(terraform output -raw ses_smtp_username 2>/dev/null)
	@SES_SMTP_PASSWORD=$$(terraform output -raw ses_smtp_password 2>/dev/null)
	@SES_EMAIL_IDENTITY=$$(terraform output -raw ses_email_identity 2>/dev/null)
	@SES_DOMAIN=$$(terraform output -raw ses_domain 2>/dev/null)
	@export FRONTEND_IMAGE="$$ECR_WEB_REPO:latest"
	@export BACKEND_IMAGE="$$ECR_API_REPO:latest"
	@export JUDGE_IMAGE="$$ECR_JUDGE_REPO:latest"
	@export SANDBOX_IMAGE="$$ECR_SANDBOX_REPO:latest"
	@export HEALER_IMAGE="$$ECR_HEALER_REPO:latest"
	@export SMTP_HOST="$$SES_SMTP_HOST"
	@export SMTP_PORT=$(SMTP_PORT)
	@export SMTP_USERNAME="$$SES_SMTP_USERNAME"
	@export SMTP_PASSWORD="$$SES_SMTP_PASSWORD"
	@export EMAIL_FROM=$$(if [ -n "$$SES_EMAIL_IDENTITY" ]; then echo "$$SES_EMAIL_IDENTITY"; elif [ -n "$$SES_DOMAIN" ]; then echo "noreply@$$SES_DOMAIN"; else echo "$(DEFAULT_EMAIL_FROM)"; fi)
	@export PUBLIC_API_URL="$(DEFAULT_PUBLIC_API_URL)"
	@envsubst $(ENV_SUBST_VARS) < k8s/apps/frontend.yaml | kubectl delete -f - --ignore-not-found
	@envsubst $(ENV_SUBST_VARS) < k8s/apps/judge.yaml | kubectl delete -f - --ignore-not-found
	@envsubst $(ENV_SUBST_VARS) < k8s/apps/healer.yaml | kubectl delete -f - --ignore-not-found
	@envsubst $(ENV_SUBST_VARS) < k8s/apps/backend.yaml | kubectl delete -f - --ignore-not-found
	@kubectl delete -f k8s/apps/surrealdb.yaml --ignore-not-found
	@kubectl delete -f k8s/addons/gp3-storageclass.yaml --ignore-not-found
	@kubectl delete -f k8s/namespaces.yaml --ignore-not-found

aws-down: k8s-down
	@helm uninstall aws-load-balancer-controller --namespace kube-system || true
	@helm uninstall metrics-server --namespace kube-system || true
	@terraform destroy -auto-approve
