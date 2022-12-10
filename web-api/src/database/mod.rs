pub mod blocks;
pub mod transactions;

use crate::config::DatabaseConfig;
use deadpool_postgres::{Config, CreatePoolError, ManagerConfig, RecyclingMethod, Runtime};
use std::ops::Deref;
use std::sync::Arc;
use tokio_postgres::NoTls;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

type Connection = deadpool_postgres::Client;

#[derive(Clone)]
pub struct Database(Arc<deadpool_postgres::Pool>);

impl Deref for Database {
    type Target = deadpool_postgres::Pool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<DatabaseConfig> for Config {
    fn from(config: DatabaseConfig) -> Self {
        let mut c = Self::new();
        c.user = Some(config.user);
        c.password = Some(config.password);
        c.host = Some(config.host);
        c.port = Some(config.port);
        c.dbname = Some(config.database);
        c.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Verified,
        });
        c
    }
}

impl Database {
    pub fn new(config: DatabaseConfig) -> std::result::Result<Self, CreatePoolError> {
        Ok(Self(Arc::new(
            Config::from(config).create_pool(Some(Runtime::Tokio1), NoTls)?,
        )))
    }
}
