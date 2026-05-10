use crate::services::auth::AuthConfig;
use crate::{config, db};
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: db::Database,
    pub auth: AuthConfig,
    pub config: Arc<config::Config>,
}
