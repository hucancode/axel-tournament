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
    # Node group for frontend, API server, and healer (balanced/lightweight workloads)
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

    # Node group for game servers (CPU-heavy workloads with Docker-in-Docker)
    game-server = {
      name           = "game-server"
      instance_types = [var.game_server_instance_type]
      desired_size   = var.game_server_desired_size
      min_size       = var.game_server_min_size
      max_size       = var.game_server_max_size
      labels = {
        role = "game-server"
      }
      taints = {
        dedicated = {
          key    = "dedicated"
          value  = "game-server"
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
