data "aws_availability_zones" "available" {
  state = "available"
}

locals {
  name = var.cluster_name
  azs  = slice(data.aws_availability_zones.available.names, 0, 2)

  public_subnets = [
    for i, _ in local.azs : cidrsubnet(var.vpc_cidr, 8, i)
  ]
  private_subnets = [
    for i, _ in local.azs : cidrsubnet(var.vpc_cidr, 8, i + 10)
  ]

  tags = merge(
    {
      Project = var.project
    },
    var.tags
  )
}

module "vpc" {
  source  = "terraform-aws-modules/vpc/aws"
  version = "~> 5.0"

  name = local.name
  cidr = var.vpc_cidr

  azs             = local.azs
  public_subnets  = local.public_subnets
  private_subnets = local.private_subnets

  enable_dns_hostnames = true
  enable_dns_support   = true

  enable_nat_gateway = true
  single_nat_gateway = true

  public_subnet_tags = {
    "kubernetes.io/cluster/${local.name}" = "shared"
    "kubernetes.io/role/elb"              = 1
  }

  private_subnet_tags = {
    "kubernetes.io/cluster/${local.name}" = "shared"
    "kubernetes.io/role/internal-elb"     = 1
  }

  tags = local.tags
}

module "eks" {
  source  = "terraform-aws-modules/eks/aws"
  version = "~> 20.0"

  cluster_name    = local.name
  cluster_version = var.kubernetes_version

  vpc_id     = module.vpc.vpc_id
  subnet_ids = module.vpc.private_subnets

  enable_irsa                               = true
  cluster_endpoint_public_access            = var.cluster_public_access_enabled
  cluster_endpoint_public_access_cidrs      = var.cluster_public_access_cidrs
  cluster_endpoint_private_access           = var.cluster_private_access_enabled
  enable_cluster_creator_admin_permissions  = true

  cluster_addons = {
    aws-ebs-csi-driver = {
      most_recent = true
      configuration_values = jsonencode({
        controller = {
          tolerations = [
            {
              key      = "dedicated"
              operator = "Equal"
              value    = "db"
              effect   = "NoSchedule"
            }
          ]
        }
        node = {
          tolerations = [
            {
              key      = "dedicated"
              operator = "Equal"
              value    = "db"
              effect   = "NoSchedule"
            }
          ]
        }
      })
    }
  }

  eks_managed_node_group_defaults = {
    ami_type       = "AL2023_x86_64_STANDARD"
    capacity_type  = "ON_DEMAND"
    disk_size      = 20
    subnet_ids     = module.vpc.private_subnets
  }

  eks_managed_node_groups = {
    app = {
      name           = "app"
      instance_types = [var.app_instance_type]
      desired_size   = var.app_desired_size
      min_size       = var.app_min_size
      max_size       = var.app_max_size
      labels = {
        role = "app"
      }
      iam_role_additional_policies = {
        ebs = "arn:aws:iam::aws:policy/service-role/AmazonEBSCSIDriverPolicy"
      }
    }

    judge = {
      name           = "judge"
      instance_types = [var.judge_instance_type]
      desired_size   = var.judge_desired_size
      min_size       = var.judge_min_size
      max_size       = var.judge_max_size
      labels = {
        role = "judge"
      }
      taints = {
        dedicated = {
          key    = "dedicated"
          value  = "judge"
          effect = "NO_SCHEDULE"
        }
      }
      iam_role_additional_policies = {
        ebs = "arn:aws:iam::aws:policy/service-role/AmazonEBSCSIDriverPolicy"
      }
    }

    db = {
      name           = "db"
      instance_types = [var.db_instance_type]
      desired_size   = var.db_desired_size
      min_size       = var.db_desired_size
      max_size       = var.db_desired_size
      labels = {
        role = "db"
      }
      taints = {
        dedicated = {
          key    = "dedicated"
          value  = "db"
          effect = "NO_SCHEDULE"
        }
      }
      iam_role_additional_policies = {
        ebs = "arn:aws:iam::aws:policy/service-role/AmazonEBSCSIDriverPolicy"
      }
    }
  }

  tags = local.tags
}
