.PHONY: aws-up aws-down k8s-up k8s-down images kustomize clean-kustomize
.ONESHELL:

# Load environment variables from .env file
-include .env
export

# Default variables
CLUSTER_NAME ?= axel-eks
SMTP_PORT ?= 587
DEFAULT_EMAIL_FROM := noreply@axel-tournament.com

# Terraform outputs (cached during make execution)
TF_ECR_WEB := $(shell terraform output -raw ecr_web_repository 2>/dev/null)
TF_ECR_API := $(shell terraform output -raw ecr_api_repository 2>/dev/null)
TF_ECR_JUDGE := $(shell terraform output -raw ecr_judge_repository 2>/dev/null)
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
	@echo "Installing EFS CSI driver..."
	@helm repo add aws-efs-csi-driver https://kubernetes-sigs.github.io/aws-efs-csi-driver/
	@helm repo update
	@helm upgrade --install aws-efs-csi-driver aws-efs-csi-driver/aws-efs-csi-driver \
		--namespace kube-system \
		--wait
	@echo "Creating EFS storage class..."
	@export EFS_FILE_SYSTEM_ID=$$(terraform output -raw efs_file_system_id 2>/dev/null || echo ""); \
	if [ -z "$$EFS_FILE_SYSTEM_ID" ]; then \
		echo "Warning: EFS filesystem not found. Skipping storage class creation."; \
	else \
		envsubst < k8s/addons/efs-storageclass.yaml | kubectl apply -f -; \
	fi
	@echo ""
	@echo "========================================="
	@echo "Infrastructure ready!"
	@echo "========================================="
	@echo ""
	@echo "Next:"
	@echo "1. Set LETSENCRYPT_EMAIL, DOMAIN and other required values in your .env file"
	@echo "2. Run 'make k8s-up' to deploy applications"

k8s-up: images kustomize
	@test -n "$(FRONTEND_IMAGE)" || (echo "FRONTEND_IMAGE is required" && exit 1)
	@test -n "$(BACKEND_IMAGE)" || (echo "BACKEND_IMAGE is required" && exit 1)
	@test -n "$(JUDGE_IMAGE)" || (echo "JUDGE_IMAGE is required" && exit 1)
	@test -n "$(GOOGLE_CLIENT_ID)" || (echo "GOOGLE_CLIENT_ID is required in .env" && exit 1)
	@test -n "$(GOOGLE_CLIENT_SECRET)" || (echo "GOOGLE_CLIENT_SECRET is required in .env" && exit 1)
	@test -n "$(DOMAIN_NAME)" || (echo "DOMAIN_NAME is required" && exit 1)
	@test -n "$(LETSENCRYPT_EMAIL)" || (echo "LETSENCRYPT_EMAIL is required in .env" && exit 1)
	@test -n "$(CERT_ISSUER)" || (echo "CERT_ISSUER is required in .env (use 'staging' or 'prod')" && exit 1)
	@echo "Validating Kustomize configuration..."
	@kubectl kustomize k8s/overlays/production > /dev/null || (echo "Kustomize validation failed" && exit 1)
	@echo "Getting NGINX LoadBalancer address..."
	@nginx_lb=$$(kubectl -n ingress-nginx get svc ingress-nginx-controller -o jsonpath='{.status.loadBalancer.ingress[0].hostname}' 2>/dev/null)
	@test -n "$$nginx_lb" || (echo "Error: NGINX LoadBalancer not found. Make sure 'make aws-up' completed successfully." && exit 1)
	echo "NGINX LoadBalancer: $$nginx_lb"
	@test -n "$(ROUTE53_ZONE_ID)" || (echo "ROUTE53_ZONE_ID is not set. Make sure 'make aws-up' completed successfully." && exit 1)
	@echo "Creating DNS records for $(DOMAIN_NAME)..."
	@nlb_zone_id=$$(aws elbv2 describe-load-balancers --query "LoadBalancers[?DNSName=='$$nginx_lb'].CanonicalHostedZoneId" --output text)
	aws route53 change-resource-record-sets \
		--hosted-zone-id $(ROUTE53_ZONE_ID) \
		--change-batch '{"Changes":[{"Action":"UPSERT","ResourceRecordSet":{"Name":"api.$(DOMAIN_NAME)","Type":"CNAME","TTL":300,"ResourceRecords":[{"Value":"'$$nginx_lb'"}]}},{"Action":"UPSERT","ResourceRecordSet":{"Name":"judge.$(DOMAIN_NAME)","Type":"CNAME","TTL":300,"ResourceRecords":[{"Value":"'$$nginx_lb'"}]}},{"Action":"UPSERT","ResourceRecordSet":{"Name":"$(DOMAIN_NAME)","Type":"A","AliasTarget":{"HostedZoneId":"'$$nlb_zone_id'","DNSName":"'$$nginx_lb'","EvaluateTargetHealth":false}}}]}'
	@echo "DNS records created. Waiting 30s for propagation..."
	@sleep 30
	@echo "Applying Kubernetes resources with Kustomize..."
	@kubectl apply -k k8s/overlays/production
	@echo "Waiting for TLS certificates to be issued..."
	@kubectl -n backend wait --for=condition=Ready certificate/backend-tls-cert --timeout=300s
	@kubectl -n backend wait --for=condition=Ready certificate/judge-tls-cert --timeout=300s
	@kubectl -n frontend wait --for=condition=Ready certificate/frontend-tls-cert --timeout=300s
	@echo "TLS certificates issued successfully"
	@echo "Waiting for backend pods to be ready..."
	@kubectl wait --namespace backend --for=condition=ready pod --selector=app=backend --timeout=120s
	@echo ""
	@echo "========================================="
	@echo "Deployment complete!"
	@echo "========================================="
	@$(MAKE) -s url

get-backend-url:
	@backend_host=$$(kubectl -n backend get ingress backend -o jsonpath='{.spec.rules[0].host}' 2>/dev/null)
	@test -n "$$backend_host" || (echo "Backend ingress not found" && exit 1)
	@echo "https://$$backend_host"

get-frontend-url:
	@frontend_host=$$(kubectl -n frontend get ingress frontend -o jsonpath='{.spec.rules[0].host}' 2>/dev/null)
	@test -n "$$frontend_host" || (echo "Frontend ingress not found" && exit 1)
	@echo "https://$$frontend_host"

url:
	@backend_url=$$($(MAKE) -s get-backend-url); \
	frontend_url=$$($(MAKE) -s get-frontend-url); \
	echo "Frontend: $$frontend_url"; \
	echo "Backend API: $$backend_url"; \
	echo ""; \
	echo "⚠️ Update your Google OAuth configuration if you haven't done so:"; \
	echo "1. Go to https://console.cloud.google.com/apis/credentials"; \
	echo "2. Edit your OAuth 2.0 Client ID"; \
	echo "3. Add this redirect URI: $$backend_url/api/auth/google/callback"; \
	domain=$$(terraform output -raw domain_name 2>/dev/null); \
	if [ -n "$$domain" ]; then \
		echo ""; \
		echo "DNS Configuration:"; \
		echo "At your domain registrar, add these NS records if you haven't done so:"; \
		echo ""; \
		terraform output -json route53_zone_name_servers 2>/dev/null | jq -r '.[] | "  - " + .' || true; \
		echo ""; \
		echo "For your domain/subdomain $$domain"; \
	fi

images:
	@test -n "$(TF_ECR_WEB)" || (echo "ECR_WEB_REPO not found. Run 'make aws-up' first." && exit 1)
	@test -n "$(TF_ECR_API)" || (echo "ECR_API_REPO not found. Run 'make aws-up' first." && exit 1)
	@test -n "$(TF_ECR_JUDGE)" || (echo "ECR_JUDGE_REPO not found. Run 'make aws-up' first." && exit 1)
	@test -n "$(DOMAIN_NAME)" || (echo "DOMAIN_NAME not found" && exit 1)
	@ECR_REGISTRY=$$(echo "$(TF_ECR_WEB)" | cut -d/ -f1); \
	echo "Logging in to ECR $$ECR_REGISTRY..."; \
	aws ecr get-login-password | docker login --username AWS --password-stdin $$ECR_REGISTRY;
	@echo "Building and pushing frontend image..."
	@docker build \
		--build-arg PUBLIC_API_URL=https://api.$(DOMAIN_NAME) \
		--build-arg PUBLIC_JUDGE_URL=https://judge.$(DOMAIN_NAME) \
		-t $(FRONTEND_IMAGE) ../web
	@docker push $(FRONTEND_IMAGE)
	@echo "Building and pushing backend image..."
	@docker build -f docker/api.Dockerfile -t $(BACKEND_IMAGE) ..
	@docker push $(BACKEND_IMAGE)
	@echo "Building and pushing unified judge image..."
	@docker build -f docker/judge.Dockerfile -t $(JUDGE_IMAGE) ..
	@docker push $(JUDGE_IMAGE)
	@echo "All images built and pushed successfully"

# Generate Kustomize overlay files from environment variables and Terraform outputs
kustomize:
	@echo "Generating Kustomize overlay files from templates..."
	@BACKEND_IMAGE_NAME=$$(echo $(BACKEND_IMAGE) | cut -d: -f1); \
	BACKEND_IMAGE_TAG=$$(echo $(BACKEND_IMAGE) | cut -d: -f2); \
	FRONTEND_IMAGE_NAME=$$(echo $(FRONTEND_IMAGE) | cut -d: -f1); \
	FRONTEND_IMAGE_TAG=$$(echo $(FRONTEND_IMAGE) | cut -d: -f2); \
	JUDGE_IMAGE_NAME=$$(echo $(JUDGE_IMAGE) | cut -d: -f1); \
	JUDGE_IMAGE_TAG=$$(echo $(JUDGE_IMAGE) | cut -d: -f2); \
	EFS_FILE_SYSTEM_ID=$$(cd . && terraform output -raw efs_file_system_id 2>/dev/null || echo "fs-placeholder"); \
	export BACKEND_IMAGE_NAME BACKEND_IMAGE_TAG FRONTEND_IMAGE_NAME FRONTEND_IMAGE_TAG JUDGE_IMAGE_NAME JUDGE_IMAGE_TAG EFS_FILE_SYSTEM_ID; \
	envsubst < k8s/overlays/production/kustomization.example.yaml > k8s/overlays/production/kustomization.yaml && \
	envsubst < k8s/overlays/production/domain-patch.example.yaml > k8s/overlays/production/domain-patch.yaml && \
	envsubst < k8s/overlays/production/efs-patch.example.yaml > k8s/overlays/production/efs-patch.yaml && \
	envsubst < k8s/overlays/production/cert-issuer-patch.example.yaml > k8s/overlays/production/cert-issuer-patch.yaml
	@echo "Kustomize overlay files generated from templates"

# Clean generated Kustomize files
clean-kustomize:
	@rm -f k8s/overlays/production/kustomization.yaml
	@rm -f k8s/overlays/production/domain-patch.yaml
	@rm -f k8s/overlays/production/efs-patch.yaml
	@rm -f k8s/overlays/production/cert-issuer-patch.yaml
	@echo "Cleaned generated Kustomize files"

k8s-down: kustomize
	@echo "Deleting Kubernetes resources with Kustomize..."
	@kubectl delete -k k8s/overlays/production --ignore-not-found

aws-down:
	@helm uninstall ingress-nginx --namespace ingress-nginx || true
	@helm uninstall cert-manager --namespace cert-manager || true
	@helm uninstall metrics-server --namespace kube-system || true
	@kubectl delete namespace ingress-nginx cert-manager --ignore-not-found || true
	@terraform destroy -auto-approve
