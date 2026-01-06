# Infrastructure

## Architecture Overview

The platform consists of the following components:

### Core Services
- **Frontend**: SvelteKit web application (namespace: `frontend`)
- **Backend API**: Rust/Axum API server (namespace: `backend`)
- **SurrealDB**: Database for storing users, tournaments, submissions, matches (namespace: `db`)
- **Healer**: Background service for cleaning up stale matches (namespace: `healer`)

### Judge Server (Unified Game Server)
The Judge server runs in the `backend` namespace and provides:

**Automated Matches (Bot vs Bot):**
- Runs tournament matches between AI submissions
- Executes user code in isolated Docker containers
- Reports results to Backend API

**Interactive Rooms (Human vs Human):**
- HTTP API for room management (`/api/rooms`)
- WebSocket connections for real-time gameplay (`/ws/{game}/{room}/{player}`)
- Sticky sessions ensure all players in same room connect to same pod
- Support for reconnection with full game state replay

**Games Supported:**
- Tic Tac Toe
- Rock Paper Scissors
- Prisoner's Dilemma

**Deployment:**
- Auto-scales from 2 to 8 replicas based on CPU
- Handles 100 concurrent rooms per pod
- 24-hour WebSocket timeout for long games

> **Note:** For detailed room management architecture, see [CHANGES.md](./CHANGES.md) and [ROOM_MANAGEMENT_DEPLOYMENT.md](./ROOM_MANAGEMENT_DEPLOYMENT.md)

## Prerequisites

Make sure you setup your **environment variables** in `.env` file:
   ```bash
   # Required
   GOOGLE_CLIENT_ID=your-google-oauth-client-id
   GOOGLE_CLIENT_SECRET=your-google-oauth-client-secret
   LETSENCRYPT_EMAIL=your-email@example.com

   # Optional (will use defaults if not set)
   AWS_REGION=us-east-1
   DOMAIN_NAME=your-domain.com
   ```

## Deployment Steps

### 1. Provision AWS Infrastructure

This creates the EKS cluster, VPC, ECR repositories, and other AWS resources:

```bash
cd infra
make aws-up
```

This will:
- Initialize Terraform
- Create EKS cluster with node groups
- Create ECR repositories for all container images
- Set up Route53 hosted zone
- Configure SES for email notifications
- Install cert-manager, NGINX ingress, and metrics-server

**Expected time**: 15-20 minutes

### 2. Configure DNS

After `make aws-up` completes, you'll see nameserver records. Add these NS records to your domain registrar:

```
example.com NS ns-123.awsdns-12.com
example.com NS ns-456.awsdns-45.net
example.com NS ns-789.awsdns-78.org
example.com NS ns-012.awsdns-01.co.uk
```

### 3. Deploy Applications

This builds all container images, pushes them to ECR, and deploys to Kubernetes:

```bash
make k8s-up
```

This will:
- Build and push all application images to ECR:
  - Frontend (web)
  - Backend API
  - Healer
  - Judge servers
- Deploy all Kubernetes resources
- Configure ingress with TLS certificates
- Set up DNS records pointing to the load balancer

_This will takes around 20-30 minutes_

### 4. Verify Deployment

Check that all pods are running:

```bash
kubectl get pods -A
```

Get your application URLs:

```bash
make url
```

### 5. Update Google OAuth Configuration

1. Go to [Google Cloud Console](https://console.cloud.google.com/apis/credentials)
2. Edit your OAuth 2.0 Client ID
3. Add these authorized redirect URIs:
   - `https://api.domain.com/api/auth/google/callback`
   - `http://localhost:8080/api/auth/google/callback` (for local development)

## Managing the Deployment

### Update Application Code

To deploy code changes:

```bash
# Build and push new images, then update deployments
make push-images

# Or just rebuild specific services
cd infra
docker build -t $(terraform output -raw ecr_api_repository):latest ../api
docker push $(terraform output -raw ecr_api_repository):latest
kubectl rollout restart deployment/backend -n backend
```

### Scale Services

Game servers auto-scale based on CPU, but you can manually adjust:

```bash
# Scale the judge server
kubectl scale deployment judge -n backend --replicas=3

# View current scaling status
kubectl get hpa -n backend
```

### View Logs

```bash
# Backend API logs
kubectl logs -f deployment/backend -n backend

# Judge server logs (with Docker-in-Docker sidecar)
kubectl logs -f deployment/judge -n backend -c judge
kubectl logs -f deployment/judge -n backend -c dockerd

# Healer logs
kubectl logs -f deployment/healer -n healer
```

### Update Secrets

```bash
# Update backend secrets
kubectl edit secret backend-secrets -n backend

# After editing, restart deployments to pick up changes
kubectl rollout restart deployment/backend -n backend
kubectl rollout restart deployment/judge -n backend
```

## Teardown

### Remove Kubernetes Resources

```bash
make k8s-down
```

### Destroy AWS Infrastructure

**Warning**: This will delete all resources including data!

```bash
make aws-down
```

### Useful Debug Informations

Download staging certificates for HTTPS testing locally
```bash
curl -o letsencrypt-staging-root.pem https://letsencrypt.org/certs/staging/letsencrypt-stg-root-x1.pem
# import the newly downloaded certificate to your favorite browser
```

```bash
# Check pod status
kubectl describe pod <pod-name> -n <namespace>

# Check events
kubectl get events -n <namespace> --sort-by='.lastTimestamp'
```

Database connection

```bash
# Check SurrealDB is running
kubectl get pods -n db

# Test connection from backend
kubectl exec -it deployment/backend -n backend -- curl http://surrealdb.db.svc.cluster.local:8000/health
```

View HPA metrics:
```bash
kubectl top pods -n backend
kubectl top nodes
```

All services expose health endpoints:
- Backend: `https://api.your-domain.com/health`
- Game servers: WebSocket ping/pong

### Database Maintainance

#### Connect to Production Database
- Database runs in private subnets only
- No direct internet access to database
- Use port-forwarding for secure access

```bash
# Forward database port to localhost
kubectl port-forward -n db svc/surrealdb 8000:8000

# In another terminal, connect with SurrealDB CLI
surreal sql --conn http://localhost:8000 --user root --pass change-me --ns tournament --db main
```

#### Database Credentials

Production credentials are stored in Kubernetes secret:
```bash
# View database credentials
kubectl get secret db-credentials -n db -o yaml
# Decode password
kubectl get secret db-credentials -n db -o jsonpath='{.data.pass}' | base64 -d
```

#### Backup and Restore

**Export Production Data:**
```bash
# With port-forward active
surreal export --conn http://localhost:8000 --user root --pass <password> --ns tournament --db main backup-$(date +%Y%m%d).sql
```

**Import to Production:**
```bash
surreal import --conn http://localhost:8000 --user root --pass <password> --ns tournament --db main backup.sql
```
