output "cluster_name" {
  value = module.eks.cluster_name
}

output "cluster_endpoint" {
  value = module.eks.cluster_endpoint
}

output "kubeconfig_command" {
  value = "aws eks update-kubeconfig --name ${module.eks.cluster_name}"
}

output "ecr_repositories" {
  value = { for name, repo in aws_ecr_repository.repos : name => repo.repository_url }
}

output "ecr_frontend_repository" {
  value = try(aws_ecr_repository.repos["frontend"].repository_url, "")
}

output "ecr_backend_repository" {
  value = try(aws_ecr_repository.repos["backend"].repository_url, "")
}

output "ecr_judge_repository" {
  value = try(aws_ecr_repository.repos["judge"].repository_url, "")
}

output "ecr_sandbox_repository" {
  value = try(aws_ecr_repository.repos["sandbox"].repository_url, "")
}
