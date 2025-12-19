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
  default     = "1.29"
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

variable "app_instance_type" {
  type        = string
  description = "Instance type for the shared app node group."
  default     = "t3.medium"
}

variable "judge_instance_type" {
  type        = string
  description = "Instance type for the judge node group."
  default     = "t3.medium"
}

variable "db_instance_type" {
  type        = string
  description = "Instance type for the database node group."
  default     = "t3.medium"
}

variable "app_desired_size" {
  type        = number
  description = "Fixed size for the app node group."
  default     = 1
}

variable "judge_desired_size" {
  type        = number
  description = "Fixed size for the judge node group."
  default     = 2
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

variable "tags" {
  type        = map(string)
  description = "Extra tags applied to all resources."
  default     = {}
}
