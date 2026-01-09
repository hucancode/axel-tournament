use crate::services::{AuthService, EmailService};
use crate::{config, db};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: db::Database,
    pub auth_service: Arc<AuthService>,
    pub email_service: Arc<EmailService>,
    pub config: Arc<config::Config>,
}
