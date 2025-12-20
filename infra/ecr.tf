locals {
  ecr_repos = ["frontend", "backend", "judge", "sandbox", "healer"]
}

resource "aws_ecr_repository" "repos" {
  for_each = var.create_ecr_repos ? toset(local.ecr_repos) : []

  name                 = "${var.project}/${each.key}"
  image_tag_mutability = "MUTABLE"

  image_scanning_configuration {
    scan_on_push = true
  }

  tags = local.tags
}
