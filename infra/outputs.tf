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

output "ecr_web_repository" {
  value = try(aws_ecr_repository.repos["web"].repository_url, "")
}

output "ecr_api_repository" {
  value = try(aws_ecr_repository.repos["api"].repository_url, "")
}

output "ecr_healer_repository" {
  value = try(aws_ecr_repository.repos["healer"].repository_url, "")
}

output "ecr_judge_repository" {
  value = try(aws_ecr_repository.repos["judge"].repository_url, "")
}

output "ses_domain_identity_arn" {
  value = try(aws_ses_domain_identity.this[0].arn, "")
}

output "ses_domain_verification_token" {
  value = try(aws_ses_domain_identity.this[0].verification_token, "")
}

output "ses_domain_dkim_tokens" {
  value = try(aws_ses_domain_dkim.this[0].dkim_tokens, [])
}

output "ses_mail_from_domain" {
  value = try(aws_ses_domain_mail_from.this[0].mail_from_domain, "")
}

output "ses_email_identity_arn" {
  value = try(aws_ses_email_identity.this[0].arn, "")
}

output "ses_email_identity" {
  value = var.ses_email_identity
}

output "ses_domain" {
  value       = local.ses_domain_enabled ? local.effective_ses_domain : ""
  description = "The domain being used for SES (either ses_domain or route53_zone_name)"
}

output "ses_smtp_host" {
  value = "email-smtp.${data.aws_region.current.name}.amazonaws.com"
}

output "ses_smtp_username" {
  value = try(aws_iam_access_key.ses_smtp[0].id, "")
}

output "ses_smtp_password" {
  value     = try(aws_iam_access_key.ses_smtp[0].ses_smtp_password_v4, "")
  sensitive = true
}

output "route53_zone_id" {
  value = aws_route53_zone.main.zone_id
}

output "route53_zone_name_servers" {
  value = aws_route53_zone.main.name_servers
}

output "domain_name" {
  description = "Domain name configured for the application"
  value       = var.route53_zone_name
}

output "aws_region" {
  value = data.aws_region.current.name
}

output "vpc_id" {
  value = module.vpc.vpc_id
}
