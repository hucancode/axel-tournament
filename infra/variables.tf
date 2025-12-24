variable "project" {
  type        = string
  description = "Project name used for tagging and resource prefixes."
  default     = "axel-tournament"
}

variable "cluster_name" {
  type        = string
  description = "EKS cluster name."
  default     = "axel-eks"
}

variable "kubernetes_version" {
  type        = string
  description = "EKS Kubernetes version."
  default     = "1.34"
}

variable "vpc_cidr" {
  type        = string
  description = "VPC CIDR block."
  default     = "10.0.0.0/16"
}

variable "cluster_public_access_cidrs" {
  type        = list(string)
  description = "CIDRs allowed to reach the EKS public API endpoint."
  default     = ["0.0.0.0/0"]
}

variable "cluster_public_access_enabled" {
  type        = bool
  description = "Whether the EKS public API endpoint is enabled."
  default     = true
}

variable "cluster_private_access_enabled" {
  type        = bool
  description = "Whether the EKS private API endpoint is enabled."
  default     = true
}

variable "app_instance_type" {
  type        = string
  description = "Instance type for the shared app node group."
  default     = "t3.small"
}

variable "judge_instance_type" {
  type        = string
  description = "Instance type for the judge node group."
  default     = "t3.small"
}

variable "db_instance_type" {
  type        = string
  description = "Instance type for the database node group."
  default     = "t3.small"
}

variable "app_desired_size" {
  type        = number
  description = "Desired size for the app node group."
  default     = 1
}

variable "app_min_size" {
  type        = number
  description = "Minimum size for the app node group."
  default     = 1
}

variable "app_max_size" {
  type        = number
  description = "Maximum size for the app node group."
  default     = 2
}

variable "judge_desired_size" {
  type        = number
  description = "Desired size for the judge node group."
  default     = 1
}

variable "judge_min_size" {
  type        = number
  description = "Minimum size for the judge node group."
  default     = 1
}

variable "judge_max_size" {
  type        = number
  description = "Maximum size for the judge node group."
  default     = 8
}

variable "db_desired_size" {
  type        = number
  description = "Fixed size for the database node group."
  default     = 1
}

variable "create_ecr_repos" {
  type        = bool
  description = "Whether to create ECR repositories for images."
  default     = true
}

variable "route53_zone_name" {
  type        = string
  description = "Route53 hosted zone name for the application domain (required)."
}

variable "ses_domain" {
  type        = string
  description = "Domain to verify in SES (leave empty to use route53_zone_name, or skip if that's also empty)."
  default     = ""
}

variable "ses_email_identity" {
  type        = string
  description = "Email address to verify in SES (leave empty to skip)."
  default     = ""
}

variable "ses_mail_from_subdomain" {
  type        = string
  description = "MAIL FROM subdomain for SES (used with ses_domain). Leave empty to skip."
  default     = ""
}

variable "create_ses_smtp_user" {
  type        = bool
  description = "Whether to create an IAM user for SES SMTP credentials."
  default     = false
}

variable "ses_smtp_user_name" {
  type        = string
  description = "IAM user name for SES SMTP credentials."
  default     = "axel-ses-smtp"
}

variable "tags" {
  type        = map(string)
  description = "Extra tags applied to all resources."
  default     = {}
}
