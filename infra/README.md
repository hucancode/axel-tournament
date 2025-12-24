# Infrastructure

## Deployment

Deploy code to productions can be as easy as
```bash
# update terraform.tfvars if you need customization
# update .env if you haven't done so
make aws-up
make k8s-up
```
After a successful deployment, you would be able to run
```bash
make url
kubectl get nodes
```

## Production Database Access

### Prerequisites

1. Configure kubectl for your EKS cluster:
```bash
aws eks update-kubeconfig --region <region> --name axel-eks
```

2. Verify cluster access:
```bash
kubectl get nodes
```

### Connect to Production Database
- Database runs in private subnets only
- No direct internet access to database
- Use port-forwarding for secure access

```bash
# Forward database port to localhost
kubectl port-forward -n db svc/surrealdb 8000:8000

# In another terminal, connect with SurrealDB CLI
surreal sql --conn http://localhost:8000 --user root --pass change-me --ns tournament --db main
```

### Database Credentials

Production credentials are stored in Kubernetes secret:
```bash
# View database credentials
kubectl get secret db-credentials -n db -o yaml
# Decode password
kubectl get secret db-credentials -n db -o jsonpath='{.data.pass}' | base64 -d
```

### Backup and Restore

**Export Production Data:**
```bash
# With port-forward active
surreal export --conn http://localhost:8000 --user root --pass <password> --ns tournament --db main backup-$(date +%Y%m%d).sql
```

**Import to Production:**
```bash
surreal import --conn http://localhost:8000 --user root --pass <password> --ns tournament --db main backup.sql
```
