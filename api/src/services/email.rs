use crate::{
    config::EmailConfig,
    error::{ApiError, ApiResult},
};
use lettre::{
    Message, SmtpTransport, message::header::ContentType,
    transport::smtp::authentication::Credentials,
    Transport,
};

pub struct EmailService {
    config: EmailConfig,
}

impl EmailService {
    pub fn new(config: EmailConfig) -> Self {
        Self { config }
    }
    pub async fn send_password_reset(&self, to_email: &str, reset_token: &str) -> ApiResult<()> {
        if self.config.smtp_username.trim().is_empty()
            || self.config.smtp_password.trim().is_empty()
        {
            return Err(ApiError::Internal(
                "SMTP credentials are not configured".to_string(),
            ));
        }
        let base_url = self.config.frontend_url.trim_end_matches('/');
        let reset_link = format!("{}/reset-password/confirm?token={}", base_url, reset_token);
        let email_body = format!(
            "Hello,\n\nYou requested a password reset. Click the link below to reset your password:\n\n{}\n\nThis link will expire in 1 hour.\n\nIf you didn't request this, please ignore this email.\n\nBest regards,\nAxel Tournament Team",
            reset_link
        );
        let email = Message::builder()
            .from(
                self.config
                    .from_address
                    .parse()
                    .map_err(|e| ApiError::Internal(format!("Invalid from address: {}", e)))?,
            )
            .to(to_email.parse().map_err(|e| {
                ApiError::Internal(format!("Invalid recipient address: {}", e))
            })?)
            .subject("Password Reset Request")
            .header(ContentType::TEXT_PLAIN)
            .body(email_body)
            .map_err(|e| ApiError::Internal(format!("Failed to build email: {}", e)))?;
        let creds = Credentials::new(
            self.config.smtp_username.clone(),
            self.config.smtp_password.clone(),
        );
        let mailer = SmtpTransport::relay(&self.config.smtp_host)
            .map_err(|e| ApiError::Internal(format!("Failed to build SMTP transport: {}", e)))?
            .port(self.config.smtp_port)
            .credentials(creds)
            .build();
        mailer.send(&email).map_err(|e| {
            ApiError::Internal(format!("Failed to send email: {}", e))
        })?;
        Ok(())
    }
}
