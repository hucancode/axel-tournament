pub mod config;
pub mod db;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod models;
pub mod services;
pub mod router;

use services::{AuthService, EmailService};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: db::Database,
    pub auth_service: Arc<AuthService>,
    pub email_service: Arc<EmailService>,
    pub config: Arc<config::Config>,
}

// Router creation is in main.rs to avoid handler signature issues with middleware
// Tests focus on business logic rather than HTTP layer
