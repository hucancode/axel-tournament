# ACM Certificate for HTTPS (only created if custom domain is set)
locals {
  domain_enabled = trimspace(var.route53_zone_name) != ""
}

resource "aws_acm_certificate" "main" {
  count = local.domain_enabled ? 1 : 0

  domain_name               = var.route53_zone_name
  subject_alternative_names = ["*.${var.route53_zone_name}"]
  validation_method         = "DNS"

  lifecycle {
    create_before_destroy = true
  }

  tags = merge(
    local.tags,
    {
      Name = "${var.route53_zone_name}-cert"
    }
  )
}

# DNS validation records for ACM
resource "aws_route53_record" "cert_validation" {
  for_each = local.domain_enabled ? {
    for dvo in aws_acm_certificate.main[0].domain_validation_options : dvo.domain_name => {
      name   = dvo.resource_record_name
      record = dvo.resource_record_value
      type   = dvo.resource_record_type
    }
  } : {}

  allow_overwrite = true
  name            = each.value.name
  records         = [each.value.record]
  ttl             = 60
  type            = each.value.type
  zone_id         = var.create_route53_zone ? aws_route53_zone.ses_subdomain[0].zone_id : var.route53_zone_id
}

# Wait for certificate validation
resource "aws_acm_certificate_validation" "main" {
  count = local.domain_enabled ? 1 : 0

  certificate_arn         = aws_acm_certificate.main[0].arn
  validation_record_fqdns = [for record in aws_route53_record.cert_validation : record.fqdn]
}
