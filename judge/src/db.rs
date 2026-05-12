use anyhow::Result;
use axel_core::db::DbConnectConfig;
pub use axel_core::db::Database;

pub async fn connect(
    url: &str,
    namespace: &str,
    database: &str,
    user: &str,
    pass: &str,
) -> Result<Database> {
    axel_core::db::connect(DbConnectConfig {
        url,
        user,
        pass,
        namespace,
        database,
    })
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))
}
