data "aws_region" "current" {}

data "aws_caller_identity" "current" {}

locals {
  # Use ses_domain if provided, otherwise fallback to route53_zone_name
  effective_ses_domain = trimspace(var.ses_domain) != "" ? var.ses_domain : var.route53_zone_name
  ses_domain_enabled   = trimspace(local.effective_ses_domain) != ""
  ses_email_enabled    = trimspace(var.ses_email_identity) != ""
  route53_zone_id      = var.create_route53_zone ? aws_route53_zone.ses_subdomain[0].zone_id : var.route53_zone_id
  route53_enabled      = var.create_route53_zone || trimspace(var.route53_zone_id) != ""
  ses_identity_arns = compact([
    local.ses_domain_enabled
    ? "arn:aws:ses:${data.aws_region.current.name}:${data.aws_caller_identity.current.account_id}:identity/${local.effective_ses_domain}"
    : "",
    local.ses_email_enabled
    ? "arn:aws:ses:${data.aws_region.current.name}:${data.aws_caller_identity.current.account_id}:identity/${var.ses_email_identity}"
    : "",
  ])
  ses_policy_resources = length(local.ses_identity_arns) > 0 ? local.ses_identity_arns : ["*"]
  ses_mail_from_domain = local.ses_domain_enabled && trimspace(var.ses_mail_from_subdomain) != "" ? "${var.ses_mail_from_subdomain}.${local.effective_ses_domain}" : ""
}

resource "aws_ses_domain_identity" "this" {
  count  = local.ses_domain_enabled ? 1 : 0
  domain = local.effective_ses_domain
}

resource "aws_ses_domain_dkim" "this" {
  count  = local.ses_domain_enabled ? 1 : 0
  domain = aws_ses_domain_identity.this[0].domain
}

resource "aws_ses_domain_mail_from" "this" {
  count            = local.ses_mail_from_domain != "" ? 1 : 0
  domain           = aws_ses_domain_identity.this[0].domain
  mail_from_domain = local.ses_mail_from_domain

  behavior_on_mx_failure = "UseDefaultValue"
}

resource "aws_ses_email_identity" "this" {
  count = local.ses_email_enabled ? 1 : 0
  email = var.ses_email_identity
}

resource "aws_route53_record" "ses_domain_verification" {
  count = local.route53_enabled && local.ses_domain_enabled ? 1 : 0

  zone_id = local.route53_zone_id
  name    = "_amazonses.${local.effective_ses_domain}"
  type    = "TXT"
  ttl     = 600
  records = [aws_ses_domain_identity.this[0].verification_token]
}

resource "aws_route53_record" "ses_dkim" {
  count = local.route53_enabled && local.ses_domain_enabled ? 3 : 0

  zone_id = local.route53_zone_id
  name    = "${aws_ses_domain_dkim.this[0].dkim_tokens[count.index]}._domainkey.${local.effective_ses_domain}"
  type    = "CNAME"
  ttl     = 600
  records = ["${aws_ses_domain_dkim.this[0].dkim_tokens[count.index]}.dkim.amazonses.com"]
}

resource "aws_route53_record" "ses_mail_from_mx" {
  count = local.route53_enabled && local.ses_mail_from_domain != "" ? 1 : 0

  zone_id = local.route53_zone_id
  name    = local.ses_mail_from_domain
  type    = "MX"
  ttl     = 600
  records = ["10 feedback-smtp.${data.aws_region.current.name}.amazonses.com"]
}

resource "aws_route53_record" "ses_mail_from_txt" {
  count = local.route53_enabled && local.ses_mail_from_domain != "" ? 1 : 0

  zone_id = local.route53_zone_id
  name    = local.ses_mail_from_domain
  type    = "TXT"
  ttl     = 600
  records = ["v=spf1 include:amazonses.com -all"]
}

resource "aws_iam_user" "ses_smtp" {
  count = var.create_ses_smtp_user ? 1 : 0
  name  = var.ses_smtp_user_name
  tags  = local.tags
}

resource "aws_iam_user_policy" "ses_smtp" {
  count = var.create_ses_smtp_user ? 1 : 0
  user  = aws_iam_user.ses_smtp[0].name
  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "ses:SendEmail",
          "ses:SendRawEmail",
        ]
        Resource = local.ses_policy_resources
      },
    ]
  })
}

resource "aws_iam_access_key" "ses_smtp" {
  count = var.create_ses_smtp_user ? 1 : 0
  user  = aws_iam_user.ses_smtp[0].name
}
