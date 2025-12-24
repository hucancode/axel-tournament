resource "aws_route53_zone" "main" {
  name = var.route53_zone_name
  tags = local.tags
}
