pub mod config;
pub mod models;
pub mod operations;
pub mod output;
pub mod sqlite;
pub mod store;

use crate::error::Result;
use config::IpamConfig;
use std::sync::Arc;
use store::IpamStore;

pub async fn create_store(config: &IpamConfig, cli_db: Option<&str>) -> Result<Arc<dyn IpamStore>> {
    let db_path = config::resolve_db_path(cli_db, &config.sqlite);
    let store = sqlite::SqliteStore::new(&db_path)?;
    store.initialize().await?;
    store.migrate().await?;
    Ok(Arc::new(store))
}
