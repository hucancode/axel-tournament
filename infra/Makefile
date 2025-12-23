.PHONY: aws-up aws-down k8s-up k8s-down push-images
.ONESHELL:

# Independent variables
CLUSTER_NAME ?= axel-eks
CONTAINER_RUNTIME ?= docker
SMTP_PORT := 587
DEFAULT_EMAIL_FROM := noreply@axel-tournament.com
DEFAULT_PUBLIC_API_URL := http://backend.backend.svc.cluster.local:8080

aws-up:
	@terraform init
	@terraform apply -auto-approve
	@aws eks update-kubeconfig --name $(CLUSTER_NAME)
	@helm repo add eks https://aws.github.io/eks-charts
	@helm repo add metrics-server https://kubernetes-sigs.github.io/metrics-server/
	@helm repo update
	@LB_CONTROLLER_ROLE_ARN=$$(terraform output -raw load_balancer_controller_role_arn 2>/dev/null)
	@test -n "$$LB_CONTROLLER_ROLE_ARN" || (echo "LB_CONTROLLER_ROLE_ARN is required" && exit 1)
	@export LB_CONTROLLER_ROLE_ARN
	@envsubst < k8s/addons/aws-load-balancer-controller-service-account.yaml | kubectl apply -f -
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

k8s-up: push-images
	@set -a; \
	[ -f .env ] && . ./.env; \
	set +a; \
	ECR_WEB_REPO=$$(terraform output -raw ecr_web_repository 2>/dev/null); \
	ECR_API_REPO=$$(terraform output -raw ecr_api_repository 2>/dev/null); \
	ECR_JUDGE_REPO=$$(terraform output -raw ecr_judge_repository 2>/dev/null); \
	ECR_SANDBOX_REPO=$$(terraform output -raw ecr_sandbox_repository 2>/dev/null); \
	ECR_HEALER_REPO=$$(terraform output -raw ecr_healer_repository 2>/dev/null); \
	SES_SMTP_HOST=$$(terraform output -raw ses_smtp_host 2>/dev/null); \
	SES_SMTP_USERNAME=$$(terraform output -raw ses_smtp_username 2>/dev/null); \
	SES_SMTP_PASSWORD=$$(terraform output -raw ses_smtp_password 2>/dev/null); \
	SES_EMAIL_IDENTITY=$$(terraform output -raw ses_email_identity 2>/dev/null); \
	SES_DOMAIN=$$(terraform output -raw ses_domain 2>/dev/null); \
	ACM_CERT_ARN=$$(terraform output -raw acm_certificate_arn 2>/dev/null); \
	DOMAIN=$$(terraform output -raw domain_name 2>/dev/null); \
	ROUTE53_ZONE_ID=$$(terraform output -raw route53_zone_id 2>/dev/null); \
	export FRONTEND_IMAGE="$$ECR_WEB_REPO:latest"; \
	export BACKEND_IMAGE="$$ECR_API_REPO:latest"; \
	export JUDGE_IMAGE="$$ECR_JUDGE_REPO:latest"; \
	export SANDBOX_IMAGE="$$ECR_SANDBOX_REPO:latest"; \
	export HEALER_IMAGE="$$ECR_HEALER_REPO:latest"; \
	export SMTP_HOST="$$SES_SMTP_HOST"; \
	export SMTP_PORT=$(SMTP_PORT); \
	export SMTP_USERNAME="$$SES_SMTP_USERNAME"; \
	export SMTP_PASSWORD="$$SES_SMTP_PASSWORD"; \
	export EMAIL_FROM=$$(if [ -n "$$SES_EMAIL_IDENTITY" ]; then echo "$$SES_EMAIL_IDENTITY"; elif [ -n "$$SES_DOMAIN" ]; then echo "noreply@$$SES_DOMAIN"; else echo "$(DEFAULT_EMAIL_FROM)"; fi); \
	export PUBLIC_API_URL="$(DEFAULT_PUBLIC_API_URL)"; \
	export GOOGLE_CLIENT_ID="$${GOOGLE_CLIENT_ID}"; \
	export GOOGLE_CLIENT_SECRET="$${GOOGLE_CLIENT_SECRET}"; \
	export ACM_CERTIFICATE_ARN="$$ACM_CERT_ARN"; \
	export DOMAIN_NAME="$$DOMAIN"; \
	test -n "$$FRONTEND_IMAGE" || (echo "FRONTEND_IMAGE is required" && exit 1); \
	test -n "$$BACKEND_IMAGE" || (echo "BACKEND_IMAGE is required" && exit 1); \
	test -n "$$JUDGE_IMAGE" || (echo "JUDGE_IMAGE is required" && exit 1); \
	test -n "$$SANDBOX_IMAGE" || (echo "SANDBOX_IMAGE is required" && exit 1); \
	test -n "$$HEALER_IMAGE" || (echo "HEALER_IMAGE is required" && exit 1); \
	test -n "$$GOOGLE_CLIENT_ID" || (echo "GOOGLE_CLIENT_ID is required" && exit 1); \
	test -n "$$GOOGLE_CLIENT_SECRET" || (echo "GOOGLE_CLIENT_SECRET is required" && exit 1)
	@kubectl apply -f k8s/namespaces.yaml
	@kubectl apply -f k8s/addons/gp3-storageclass.yaml
	@kubectl apply -f k8s/apps/surrealdb.yaml
	@echo "Deploying applications (initial pass with placeholder URLs)..."
	@set -a; \
	[ -f .env ] && . ./.env; \
	set +a; \
	export GOOGLE_REDIRECT_URI="http://placeholder/api/auth/google/callback"; \
	export PUBLIC_API_URL="http://placeholder"; \
	export FRONTEND_URL="http://placeholder"; \
	envsubst < k8s/apps/backend.yaml | kubectl apply -f -; \
	envsubst < k8s/apps/judge.yaml | kubectl apply -f -; \
	envsubst < k8s/apps/healer.yaml | kubectl apply -f -; \
	envsubst < k8s/apps/frontend.yaml | kubectl apply -f -
	@set -a; \
	[ -f .env ] && . ./.env; \
	set +a; \
	ACM_CERT_ARN=$$(terraform output -raw acm_certificate_arn 2>/dev/null); \
	DOMAIN=$$(terraform output -raw domain_name 2>/dev/null); \
	export ACM_CERTIFICATE_ARN="$$ACM_CERT_ARN"; \
	export DOMAIN_NAME="$$DOMAIN"; \
	if [ -n "$$DOMAIN_NAME" ]; then \
		echo "Using domain-based ingress"; \
		if [ -n "$$ACM_CERTIFICATE_ARN" ]; then \
			envsubst < k8s/apps/ingress/backend-https-domain.yaml | kubectl apply -f -; \
			envsubst < k8s/apps/ingress/frontend-https-domain.yaml | kubectl apply -f -; \
		else \
			envsubst < k8s/apps/ingress/backend-http-domain.yaml | kubectl apply -f -; \
			envsubst < k8s/apps/ingress/frontend-http-domain.yaml | kubectl apply -f -; \
		fi; \
	else \
		echo "Using ALB hostname ingress"; \
		if [ -n "$$ACM_CERTIFICATE_ARN" ]; then \
			envsubst < k8s/apps/ingress/backend-https.yaml | kubectl apply -f -; \
			envsubst < k8s/apps/ingress/frontend-https.yaml | kubectl apply -f -; \
		else \
			envsubst < k8s/apps/ingress/backend-http.yaml | kubectl apply -f -; \
			envsubst < k8s/apps/ingress/frontend-http.yaml | kubectl apply -f -; \
		fi; \
	fi
	@echo "Waiting for ALB load balancers to be provisioned..."
	@set -a; \
	[ -f .env ] && . ./.env; \
	set +a; \
	backend_alb=""; \
	frontend_alb=""; \
	for i in $$(seq 1 30); do \
		backend_alb=$$(kubectl -n backend get ingress backend -o jsonpath='{.status.loadBalancer.ingress[0].hostname}' 2>/dev/null); \
		frontend_alb=$$(kubectl -n frontend get ingress frontend -o jsonpath='{.status.loadBalancer.ingress[0].hostname}' 2>/dev/null); \
		if [ -n "$$backend_alb" ] && [ -n "$$frontend_alb" ]; then \
			echo "ALBs provisioned successfully"; \
			break; \
		fi; \
		echo "Waiting for ALBs... ($$i/30)"; \
		sleep 10; \
	done; \
	if [ -z "$$backend_alb" ] || [ -z "$$frontend_alb" ]; then \
		echo "Error: ALB hostnames not available after 5 minutes"; \
		exit 1; \
	fi; \
	echo "Backend ALB: $$backend_alb"; \
	echo "Frontend ALB: $$frontend_alb"; \
	if [ -n "$$DOMAIN_NAME" ] && [ -n "$$ROUTE53_ZONE_ID" ]; then \
		echo "Custom domain detected: $$DOMAIN_NAME"; \
		echo "Creating DNS records..."; \
		aws route53 change-resource-record-sets --hosted-zone-id $$ROUTE53_ZONE_ID --change-batch '{ \
			"Changes": [ \
				{ \
					"Action": "UPSERT", \
					"ResourceRecordSet": { \
						"Name": "api.'"$$DOMAIN_NAME"'", \
						"Type": "CNAME", \
						"TTL": 300, \
						"ResourceRecords": [{"Value": "'"$$backend_alb"'"}] \
					} \
				}, \
				{ \
					"Action": "UPSERT", \
					"ResourceRecordSet": { \
						"Name": "'"$$DOMAIN_NAME"'", \
						"Type": "CNAME", \
						"TTL": 300, \
						"ResourceRecords": [{"Value": "'"$$frontend_alb"'"}] \
					} \
				} \
			] \
		}'; \
	fi; \
	backend_url=$$($(MAKE) -s get-backend-url); \
	frontend_url=$$($(MAKE) -s get-frontend-url); \
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
	@echo ""
	@echo "========================================="

get-backend-url:
	@backend_host=$$(kubectl -n backend get ingress backend -o jsonpath='{.spec.rules[0].host}' 2>/dev/null); \
	backend_alb=$$(kubectl -n backend get ingress backend -o jsonpath='{.status.loadBalancer.ingress[0].hostname}' 2>/dev/null); \
	has_cert=$$(kubectl -n backend get ingress backend -o jsonpath='{.metadata.annotations.alb\.ingress\.kubernetes\.io/certificate-arn}' 2>/dev/null); \
	if [ -n "$$backend_host" ]; then \
		if [ -n "$$has_cert" ]; then \
			echo "https://$$backend_host"; \
		else \
			echo "http://$$backend_host"; \
		fi; \
	else \
		echo "http://$$backend_alb"; \
	fi

get-frontend-url:
	@frontend_host=$$(kubectl -n frontend get ingress frontend -o jsonpath='{.spec.rules[0].host}' 2>/dev/null); \
	frontend_alb=$$(kubectl -n frontend get ingress frontend -o jsonpath='{.status.loadBalancer.ingress[0].hostname}' 2>/dev/null); \
	has_cert=$$(kubectl -n frontend get ingress frontend -o jsonpath='{.metadata.annotations.alb\.ingress\.kubernetes\.io/certificate-arn}' 2>/dev/null); \
	if [ -n "$$frontend_host" ]; then \
		if [ -n "$$has_cert" ]; then \
			echo "https://$$frontend_host"; \
		else \
			echo "http://$$frontend_host"; \
		fi; \
	else \
		echo "http://$$frontend_alb"; \
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
		echo "Make sure your domain's nameservers point to:"; \
		terraform output -raw route53_zone_name_servers; \
	fi

push-images:
	@ECR_WEB_REPO=$$(terraform output -raw ecr_web_repository 2>/dev/null); \
	ECR_API_REPO=$$(terraform output -raw ecr_api_repository 2>/dev/null); \
	ECR_JUDGE_REPO=$$(terraform output -raw ecr_judge_repository 2>/dev/null); \
	ECR_SANDBOX_REPO=$$(terraform output -raw ecr_sandbox_repository 2>/dev/null); \
	ECR_HEALER_REPO=$$(terraform output -raw ecr_healer_repository 2>/dev/null); \
	test -n "$$ECR_WEB_REPO" || (echo "ECR_WEB_REPO is required" && exit 1); \
	test -n "$$ECR_API_REPO" || (echo "ECR_API_REPO is required" && exit 1); \
	test -n "$$ECR_JUDGE_REPO" || (echo "ECR_JUDGE_REPO is required" && exit 1); \
	test -n "$$ECR_SANDBOX_REPO" || (echo "ECR_SANDBOX_REPO is required" && exit 1); \
	test -n "$$ECR_HEALER_REPO" || (echo "ECR_HEALER_REPO is required" && exit 1); \
	FRONTEND_IMAGE="$$ECR_WEB_REPO:latest"; \
	BACKEND_IMAGE="$$ECR_API_REPO:latest"; \
	JUDGE_IMAGE="$$ECR_JUDGE_REPO:latest"; \
	SANDBOX_IMAGE="$$ECR_SANDBOX_REPO:latest"; \
	HEALER_IMAGE="$$ECR_HEALER_REPO:latest"; \
	ECR_REGISTRY=$$(echo "$$ECR_WEB_REPO" | cut -d/ -f1); \
	echo "Logging in to ECR $$ECR_REGISTRY..."; \
	aws ecr get-login-password | $(CONTAINER_RUNTIME) login --username AWS --password-stdin $$ECR_REGISTRY; \
	echo "Building and pushing frontend image..."; \
	$(CONTAINER_RUNTIME) build -t $$FRONTEND_IMAGE ../web; \
	$(CONTAINER_RUNTIME) push $$FRONTEND_IMAGE; \
	echo "Building and pushing backend image..."; \
	$(CONTAINER_RUNTIME) build -t $$BACKEND_IMAGE ../api; \
	$(CONTAINER_RUNTIME) push $$BACKEND_IMAGE; \
	echo "Building and pushing judge image..."; \
	$(CONTAINER_RUNTIME) build -t $$JUDGE_IMAGE ../judge; \
	$(CONTAINER_RUNTIME) push $$JUDGE_IMAGE; \
	echo "Building and pushing sandbox image..."; \
	$(CONTAINER_RUNTIME) build -f ../judge/src/sandbox.Dockerfile -t $$SANDBOX_IMAGE ../judge/src; \
	$(CONTAINER_RUNTIME) push $$SANDBOX_IMAGE; \
	echo "Building and pushing healer image..."; \
	$(CONTAINER_RUNTIME) build -t $$HEALER_IMAGE ../healer; \
	$(CONTAINER_RUNTIME) push $$HEALER_IMAGE; \
	echo "All images built and pushed successfully"

k8s-down:
	@set -a; \
	[ -f .env ] && . ./.env; \
	set +a; \
	ECR_WEB_REPO=$$(terraform output -raw ecr_web_repository 2>/dev/null); \
	ECR_API_REPO=$$(terraform output -raw ecr_api_repository 2>/dev/null); \
	ECR_JUDGE_REPO=$$(terraform output -raw ecr_judge_repository 2>/dev/null); \
	ECR_SANDBOX_REPO=$$(terraform output -raw ecr_sandbox_repository 2>/dev/null); \
	ECR_HEALER_REPO=$$(terraform output -raw ecr_healer_repository 2>/dev/null); \
	SES_SMTP_HOST=$$(terraform output -raw ses_smtp_host 2>/dev/null); \
	SES_SMTP_USERNAME=$$(terraform output -raw ses_smtp_username 2>/dev/null); \
	SES_SMTP_PASSWORD=$$(terraform output -raw ses_smtp_password 2>/dev/null); \
	SES_EMAIL_IDENTITY=$$(terraform output -raw ses_email_identity 2>/dev/null); \
	SES_DOMAIN=$$(terraform output -raw ses_domain 2>/dev/null); \
	export FRONTEND_IMAGE="$$ECR_WEB_REPO:latest"; \
	export BACKEND_IMAGE="$$ECR_API_REPO:latest"; \
	export JUDGE_IMAGE="$$ECR_JUDGE_REPO:latest"; \
	export SANDBOX_IMAGE="$$ECR_SANDBOX_REPO:latest"; \
	export HEALER_IMAGE="$$ECR_HEALER_REPO:latest"; \
	export SMTP_HOST="$$SES_SMTP_HOST"; \
	export SMTP_PORT=$(SMTP_PORT); \
	export SMTP_USERNAME="$$SES_SMTP_USERNAME"; \
	export SMTP_PASSWORD="$$SES_SMTP_PASSWORD"; \
	export EMAIL_FROM=$$(if [ -n "$$SES_EMAIL_IDENTITY" ]; then echo "$$SES_EMAIL_IDENTITY"; elif [ -n "$$SES_DOMAIN" ]; then echo "noreply@$$SES_DOMAIN"; else echo "$(DEFAULT_EMAIL_FROM)"; fi); \
	export PUBLIC_API_URL="$(DEFAULT_PUBLIC_API_URL)"; \
	export FRONTEND_URL="http://placeholder"; \
	export GOOGLE_CLIENT_ID="$${GOOGLE_CLIENT_ID:-placeholder}"; \
	export GOOGLE_CLIENT_SECRET="$${GOOGLE_CLIENT_SECRET:-placeholder}"; \
	export GOOGLE_REDIRECT_URI="$${GOOGLE_REDIRECT_URI:-placeholder}"; \
	envsubst < k8s/apps/frontend.yaml | kubectl delete -f - --ignore-not-found; \
	envsubst < k8s/apps/judge.yaml | kubectl delete -f - --ignore-not-found; \
	envsubst < k8s/apps/healer.yaml | kubectl delete -f - --ignore-not-found; \
	envsubst < k8s/apps/backend.yaml | kubectl delete -f - --ignore-not-found
	@envsubst < k8s/apps/ingress/backend-https.yaml | kubectl delete -f - --ignore-not-found
	@envsubst < k8s/apps/ingress/frontend-https.yaml | kubectl delete  -f - --ignore-not-found
	@envsubst < k8s/apps/ingress/backend-http.yaml | kubectl delete -f - --ignore-not-found
	@envsubst < k8s/apps/ingress/frontend-http.yaml | kubectl delete -f - --ignore-not-found
	@envsubst < k8s/apps/ingress/backend-https-domain.yaml | kubectl delete -f - --ignore-not-found
	@envsubst < k8s/apps/ingress/frontend-https-domain.yaml | kubectl delete  -f - --ignore-not-found
	@envsubst < k8s/apps/ingress/backend-http-domain.yaml | kubectl delete -f - --ignore-not-found
	@envsubst < k8s/apps/ingress/frontend-http-domain.yaml | kubectl delete -f - --ignore-not-found
	@kubectl delete -f k8s/apps/surrealdb.yaml --ignore-not-found
	@kubectl delete -f k8s/addons/gp3-storageclass.yaml --ignore-not-found
	@kubectl delete -f k8s/namespaces.yaml --ignore-not-found

aws-down:
	@helm uninstall aws-load-balancer-controller --namespace kube-system || true
	@helm uninstall metrics-server --namespace kube-system || true
	@terraform destroy -auto-approve
