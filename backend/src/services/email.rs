use crate::{config::EmailConfig, error::ApiResult};
use lettre::{
    message::header::ContentType, transport::smtp::authentication::Credentials, Message,
    SmtpTransport,
};

pub struct EmailService {
    config: EmailConfig,
}

impl EmailService {
    pub fn new(config: EmailConfig) -> Self {
        Self { config }
    }

    pub async fn send_password_reset(
        &self,
        to_email: &str,
        reset_token: &str,
    ) -> ApiResult<()> {
        let reset_link = format!("http://localhost:8080/reset-password?token={}", reset_token);

        let email_body = format!(
            "Hello,\n\nYou requested a password reset. Click the link below to reset your password:\n\n{}\n\nThis link will expire in 1 hour.\n\nIf you didn't request this, please ignore this email.\n\nBest regards,\nAxel Tournament Team",
            reset_link
        );

        let _email = Message::builder()
            .from(self.config.from_address.parse().unwrap())
            .to(to_email.parse().unwrap())
            .subject("Password Reset Request")
            .header(ContentType::TEXT_PLAIN)
            .body(email_body)
            .unwrap();

        let creds = Credentials::new(
            self.config.smtp_username.clone(),
            self.config.smtp_password.clone(),
        );

        let _mailer = SmtpTransport::relay(&self.config.smtp_host)
            .unwrap()
            .credentials(creds)
            .build();

        // For development, we'll just log the email instead of sending



        // Uncomment to actually send emails
        // mailer.send(&email).map_err(|e| {
        //     ApiError::Internal(format!("Failed to send email: {}", e))
        // })?;

        Ok(())
    }
}
