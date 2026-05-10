use crate::{
    config::EmailConfig,
    error::{ApiError, ApiResult},
};
use lettre::{
    Message, SmtpTransport, Transport,
    message::header::ContentType,
    transport::smtp::authentication::Credentials,
};

pub async fn send_password_reset(
    config: &EmailConfig,
    to_email: &str,
    reset_token: &str,
) -> ApiResult<()> {
    if config.smtp_username.trim().is_empty() || config.smtp_password.trim().is_empty() {
        return Err(ApiError::Email(
            "SMTP credentials are not configured".to_string(),
        ));
    }
    let base_url = config.frontend_url.trim_end_matches('/');
    let reset_link = format!("{}/reset-password/confirm?token={}", base_url, reset_token);
    let body = format!(
        "Hello,\n\nYou requested a password reset. Click the link below to reset your password:\n\n{}\n\nThis link will expire in 1 hour.\n\nIf you didn't request this, please ignore this email.\n\nBest regards,\nAxel Tournament Team",
        reset_link
    );
    let email = Message::builder()
        .from(
            config
                .from_address
                .parse()
                .map_err(|e| ApiError::Email(format!("Invalid from address: {}", e)))?,
        )
        .to(to_email
            .parse()
            .map_err(|e| ApiError::Email(format!("Invalid recipient address: {}", e)))?)
        .subject("Password Reset Request")
        .header(ContentType::TEXT_PLAIN)
        .body(body)
        .map_err(|e| ApiError::Email(format!("Failed to build email: {}", e)))?;
    let mailer = if config.smtp_use_tls {
        let creds = Credentials::new(
            config.smtp_username.clone(),
            config.smtp_password.clone(),
        );
        SmtpTransport::relay(&config.smtp_host)
            .map_err(|e| ApiError::Email(format!("Failed to build SMTP transport: {}", e)))?
            .port(config.smtp_port)
            .credentials(creds)
            .build()
    } else {
        SmtpTransport::builder_dangerous(&config.smtp_host)
            .port(config.smtp_port)
            .build()
    };
    mailer
        .send(&email)
        .map_err(|e| ApiError::Email(format!("Failed to send email: {}", e)))?;
    Ok(())
}
