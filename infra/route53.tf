resource "aws_route53_zone" "ses_subdomain" {
  count = var.create_route53_zone ? 1 : 0

  name = var.route53_zone_name
  tags = local.tags
}
