.PHONY: aws-up aws-down k8s-up k8s-down push-images
.ONESHELL:

# Load environment variables from .env file
-include .env
export

# Default variables
CLUSTER_NAME ?= axel-eks
CONTAINER_RUNTIME ?= docker
SMTP_PORT ?= 587
DEFAULT_EMAIL_FROM := noreply@axel-tournament.com

# Terraform outputs (cached during make execution)
TF_ECR_WEB := $(shell terraform output -raw ecr_web_repository 2>/dev/null)
TF_ECR_API := $(shell terraform output -raw ecr_api_repository 2>/dev/null)
TF_ECR_JUDGE := $(shell terraform output -raw ecr_judge_repository 2>/dev/null)
TF_ECR_SANDBOX := $(shell terraform output -raw ecr_sandbox_repository 2>/dev/null)
TF_ECR_HEALER := $(shell terraform output -raw ecr_healer_repository 2>/dev/null)
TF_SES_SMTP_HOST := $(shell terraform output -raw ses_smtp_host 2>/dev/null)
TF_SES_SMTP_USERNAME := $(shell terraform output -raw ses_smtp_username 2>/dev/null)
TF_SES_SMTP_PASSWORD := $(shell terraform output -raw ses_smtp_password 2>/dev/null)
TF_SES_EMAIL_IDENTITY := $(shell terraform output -raw ses_email_identity 2>/dev/null)
TF_SES_DOMAIN := $(shell terraform output -raw ses_domain 2>/dev/null)
TF_DOMAIN := $(shell terraform output -raw domain_name 2>/dev/null)
TF_ROUTE53_ZONE_ID := $(shell terraform output -raw route53_zone_id 2>/dev/null)

# Computed variables
FRONTEND_IMAGE := $(TF_ECR_WEB):latest
BACKEND_IMAGE := $(TF_ECR_API):latest
JUDGE_IMAGE := $(TF_ECR_JUDGE):latest
SANDBOX_IMAGE := $(TF_ECR_SANDBOX):latest
HEALER_IMAGE := $(TF_ECR_HEALER):latest
SMTP_HOST := $(TF_SES_SMTP_HOST)
SMTP_USERNAME := $(TF_SES_SMTP_USERNAME)
SMTP_PASSWORD := $(TF_SES_SMTP_PASSWORD)
EMAIL_FROM := $(or $(TF_SES_EMAIL_IDENTITY),$(if $(TF_SES_DOMAIN),noreply@$(TF_SES_DOMAIN),$(DEFAULT_EMAIL_FROM)))
DOMAIN_NAME := $(TF_DOMAIN)
ROUTE53_ZONE_ID := $(TF_ROUTE53_ZONE_ID)

aws-up:
	@terraform init || exit 1
	@terraform apply -auto-approve || exit 1
	@aws eks update-kubeconfig --name $(CLUSTER_NAME)
	@echo "Adding Helm repositories..."
	@helm repo add jetstack https://charts.jetstack.io
	@helm repo add ingress-nginx https://kubernetes.github.io/ingress-nginx
	@helm repo add metrics-server https://kubernetes-sigs.github.io/metrics-server/
	@helm repo update
	@echo "Installing cert-manager..."
	@helm upgrade --install cert-manager jetstack/cert-manager \
		--namespace cert-manager \
		--create-namespace \
		--version v1.16.2 \
		--values k8s/helm-values/cert-manager-values.yaml \
		--wait \
		--timeout 5m
	@echo "Installing NGINX Ingress Controller..."
	@helm upgrade --install ingress-nginx ingress-nginx/ingress-nginx \
		--namespace ingress-nginx \
		--create-namespace \
		--values k8s/helm-values/ingress-nginx-values.yaml \
		--wait \
		--timeout 5m
	@echo "Installing metrics-server..."
	@helm upgrade --install metrics-server metrics-server/metrics-server \
		--namespace kube-system \
		--set args={--kubelet-insecure-tls} \
		--wait
	@echo ""
	@echo "========================================="
	@echo "Infrastructure ready!"
	@echo "========================================="
	@echo ""
	@echo "Next:"
	@echo "1. Set LETSENCRYPT_EMAIL, DOMAIN and other required values in your .env file"
	@echo "2. Run 'make k8s-up' to deploy applications"

k8s-up: push-images
	@test -n "$(FRONTEND_IMAGE)" || (echo "FRONTEND_IMAGE is required" && exit 1)
	@test -n "$(BACKEND_IMAGE)" || (echo "BACKEND_IMAGE is required" && exit 1)
	@test -n "$(JUDGE_IMAGE)" || (echo "JUDGE_IMAGE is required" && exit 1)
	@test -n "$(SANDBOX_IMAGE)" || (echo "SANDBOX_IMAGE is required" && exit 1)
	@test -n "$(HEALER_IMAGE)" || (echo "HEALER_IMAGE is required" && exit 1)
	@test -n "$(GOOGLE_CLIENT_ID)" || (echo "GOOGLE_CLIENT_ID is required in .env" && exit 1)
	@test -n "$(GOOGLE_CLIENT_SECRET)" || (echo "GOOGLE_CLIENT_SECRET is required in .env" && exit 1)
	@test -n "$(DOMAIN_NAME)" || (echo "DOMAIN_NAME is required" && exit 1)
	@test -n "$(LETSENCRYPT_EMAIL)" || (echo "LETSENCRYPT_EMAIL is required in .env" && exit 1)
	@echo "Configuring cert-manager ClusterIssuer..."
	@envsubst < k8s/addons/cert-manager.yaml | kubectl apply -f -
	@kubectl apply -f k8s/namespaces.yaml
	@kubectl apply -f k8s/addons/gp3-storageclass.yaml
	@kubectl apply -f k8s/apps/surrealdb.yaml
	@echo "Getting NGINX LoadBalancer address..."
	@nginx_lb=$$(kubectl -n ingress-nginx get svc ingress-nginx-controller -o jsonpath='{.status.loadBalancer.ingress[0].hostname}' 2>/dev/null); \
	if [ -z "$$nginx_lb" ]; then \
		echo "Error: NGINX LoadBalancer not found. Make sure 'make aws-up' completed successfully."; \
		exit 1; \
	fi; \
	echo "NGINX LoadBalancer: $$nginx_lb"
	@nginx_lb=$$(kubectl -n ingress-nginx get svc ingress-nginx-controller -o jsonpath='{.status.loadBalancer.ingress[0].hostname}' 2>/dev/null); \
	if [ -n "$(ROUTE53_ZONE_ID)" ]; then \
		echo "Creating DNS records for $(DOMAIN_NAME)..."; \
		nlb_zone_id=$$(aws elbv2 describe-load-balancers --query "LoadBalancers[?DNSName=='$$nginx_lb'].CanonicalHostedZoneId" --output text); \
		aws route53 change-resource-record-sets \
			--hosted-zone-id $(ROUTE53_ZONE_ID) \
			--change-batch '{"Changes":[{"Action":"UPSERT","ResourceRecordSet":{"Name":"api.$(DOMAIN_NAME)","Type":"CNAME","TTL":300,"ResourceRecords":[{"Value":"'$$nginx_lb'"}]}},{"Action":"UPSERT","ResourceRecordSet":{"Name":"$(DOMAIN_NAME)","Type":"A","AliasTarget":{"HostedZoneId":"'$$nlb_zone_id'","DNSName":"'$$nginx_lb'","EvaluateTargetHealth":false}}}]}'; \
		echo "DNS records created. Waiting 30s for propagation..."; \
		sleep 30; \
	else \
		echo ""; \
		echo "⚠️  IMPORTANT: Point your DNS to the NGINX LoadBalancer:"; \
		echo "  api.$(DOMAIN_NAME) → $$nginx_lb (CNAME)"; \
		echo "  $(DOMAIN_NAME) → $$nginx_lb (A/ALIAS)"; \
		echo ""; \
		read -p "Press Enter once DNS is configured..."; \
	fi
	@echo "Deploying applications (initial pass with placeholder URLs)..."
	@export GOOGLE_REDIRECT_URI="http://placeholder/api/auth/google/callback"; \
	export PUBLIC_API_URL="http://placeholder"; \
	export FRONTEND_URL="http://placeholder"; \
	envsubst < k8s/apps/backend.yaml | kubectl apply -f -; \
	envsubst < k8s/apps/judge.yaml | kubectl apply -f -; \
	envsubst < k8s/apps/healer.yaml | kubectl apply -f -; \
	envsubst < k8s/apps/frontend.yaml | kubectl apply -f -
	@echo "Deploying ingress resources..."
	@envsubst < k8s/apps/ingress/backend.yaml | kubectl apply -f -
	@envsubst < k8s/apps/ingress/frontend.yaml | kubectl apply -f -
	@echo "Waiting for TLS certificates to be issued..."
	@kubectl -n backend wait --for=condition=Ready certificate/backend-tls-cert --timeout=300s
	@kubectl -n frontend wait --for=condition=Ready certificate/frontend-tls-cert --timeout=300s
	@echo "TLS certificates issued successfully"
	@backend_url="https://api.$(DOMAIN_NAME)"; \
	frontend_url="https://$(DOMAIN_NAME)"; \
	export GOOGLE_REDIRECT_URI="$$backend_url/api/auth/google/callback"; \
	export PUBLIC_API_URL="$$backend_url"; \
	export FRONTEND_URL="$$frontend_url"; \
	echo "Final URLs:"; \
	echo "  Frontend: $$frontend_url"; \
	echo "  Backend: $$backend_url"; \
	echo "  OAuth Redirect: $$GOOGLE_REDIRECT_URI"; \
	echo "Updating deployments with final URLs..."; \
	envsubst < k8s/apps/backend.yaml | kubectl apply -f -; \
	envsubst < k8s/apps/judge.yaml | kubectl apply -f -; \
	envsubst < k8s/apps/healer.yaml | kubectl apply -f -; \
	envsubst < k8s/apps/frontend.yaml | kubectl apply -f -
	@echo "Waiting for backend pods to be ready..."
	@kubectl wait --namespace backend --for=condition=ready pod --selector=app=backend --timeout=120s || true
	@echo ""
	@echo "========================================="
	@echo "Deployment complete!"
	@echo "========================================="
	@$(MAKE) -s print-urls

get-backend-url:
	@backend_host=$$(kubectl -n backend get ingress backend -o jsonpath='{.spec.rules[0].host}' 2>/dev/null); \
	if [ -n "$$backend_host" ]; then \
		echo "https://$$backend_host"; \
	else \
		echo "Backend ingress not found"; \
		exit 1; \
	fi

get-frontend-url:
	@frontend_host=$$(kubectl -n frontend get ingress frontend -o jsonpath='{.spec.rules[0].host}' 2>/dev/null); \
	if [ -n "$$frontend_host" ]; then \
		echo "https://$$frontend_host"; \
	else \
		echo "Frontend ingress not found"; \
		exit 1; \
	fi

print-urls:
	@backend_url=$$($(MAKE) -s get-backend-url); \
	frontend_url=$$($(MAKE) -s get-frontend-url); \
	echo "Frontend: $$frontend_url"; \
	echo "Backend API: $$backend_url"; \
	echo "Google OAuth Redirect URI: $$backend_url/api/auth/google/callback"; \
	echo ""; \
	echo "⚠️  IMPORTANT: Update your Google OAuth configuration:"; \
	echo "1. Go to https://console.cloud.google.com/apis/credentials"; \
	echo "2. Edit your OAuth 2.0 Client ID"; \
	echo "3. Add this redirect URI: $$backend_url/api/auth/google/callback"; \
	domain=$$(terraform output -raw domain_name 2>/dev/null); \
	if [ -n "$$domain" ]; then \
		echo ""; \
		echo "📋 DNS Configuration:"; \
		echo "At your domain registrar for the parent domain, add these NS records:"; \
		echo ""; \
		terraform output -json route53_zone_name_servers 2>/dev/null | jq -r '.[] | "  - " + .' || true; \
		echo ""; \
		echo "For subdomain 'axel' under your parent domain"; \
	fi

push-images:
	@test -n "$(TF_ECR_WEB)" || (echo "ECR_WEB_REPO not found. Run 'make aws-up' first." && exit 1)
	@test -n "$(TF_ECR_API)" || (echo "ECR_API_REPO not found. Run 'make aws-up' first." && exit 1)
	@test -n "$(TF_ECR_JUDGE)" || (echo "ECR_JUDGE_REPO not found. Run 'make aws-up' first." && exit 1)
	@test -n "$(TF_ECR_SANDBOX)" || (echo "ECR_SANDBOX_REPO not found. Run 'make aws-up' first." && exit 1)
	@test -n "$(TF_ECR_HEALER)" || (echo "ECR_HEALER_REPO not found. Run 'make aws-up' first." && exit 1)
	@ECR_REGISTRY=$$(echo "$(TF_ECR_WEB)" | cut -d/ -f1); \
	echo "Logging in to ECR $$ECR_REGISTRY..."; \
	aws ecr get-login-password | $(CONTAINER_RUNTIME) login --username AWS --password-stdin $$ECR_REGISTRY; \
	echo "Building and pushing frontend image..."; \
	$(CONTAINER_RUNTIME) build -t $(FRONTEND_IMAGE) ../web; \
	$(CONTAINER_RUNTIME) push $(FRONTEND_IMAGE); \
	echo "Building and pushing backend image..."; \
	$(CONTAINER_RUNTIME) build -t $(BACKEND_IMAGE) ../api; \
	$(CONTAINER_RUNTIME) push $(BACKEND_IMAGE); \
	echo "Building and pushing judge image..."; \
	$(CONTAINER_RUNTIME) build -t $(JUDGE_IMAGE) ../judge; \
	$(CONTAINER_RUNTIME) push $(JUDGE_IMAGE); \
	echo "Building and pushing sandbox image..."; \
	$(CONTAINER_RUNTIME) build -f ../judge/src/sandbox.Dockerfile -t $(SANDBOX_IMAGE) ../judge/src; \
	$(CONTAINER_RUNTIME) push $(SANDBOX_IMAGE); \
	echo "Building and pushing healer image..."; \
	$(CONTAINER_RUNTIME) build -t $(HEALER_IMAGE) ../healer; \
	$(CONTAINER_RUNTIME) push $(HEALER_IMAGE); \
	echo "All images built and pushed successfully"

k8s-down:
	@export GOOGLE_CLIENT_ID="$${GOOGLE_CLIENT_ID:-placeholder}"; \
	export GOOGLE_CLIENT_SECRET="$${GOOGLE_CLIENT_SECRET:-placeholder}"; \
	export GOOGLE_REDIRECT_URI="placeholder"; \
	export PUBLIC_API_URL="placeholder"; \
	export FRONTEND_URL="placeholder"; \
	envsubst < k8s/apps/frontend.yaml | kubectl delete -f - --ignore-not-found; \
	envsubst < k8s/apps/judge.yaml | kubectl delete -f - --ignore-not-found; \
	envsubst < k8s/apps/healer.yaml | kubectl delete -f - --ignore-not-found; \
	envsubst < k8s/apps/backend.yaml | kubectl delete -f - --ignore-not-found
	@envsubst < k8s/apps/ingress/backend.yaml | kubectl delete -f - --ignore-not-found
	@envsubst < k8s/apps/ingress/frontend.yaml | kubectl delete -f - --ignore-not-found
	@kubectl delete -f k8s/apps/surrealdb.yaml --ignore-not-found
	@kubectl delete -f k8s/addons/gp3-storageclass.yaml --ignore-not-found
	@kubectl delete -f k8s/namespaces.yaml --ignore-not-found

aws-down:
	@helm uninstall ingress-nginx --namespace ingress-nginx || true
	@helm uninstall cert-manager --namespace cert-manager || true
	@helm uninstall metrics-server --namespace kube-system || true
	@kubectl delete namespace ingress-nginx cert-manager --ignore-not-found || true
	@terraform destroy -auto-approve
