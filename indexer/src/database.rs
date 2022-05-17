use crate::DatabaseConfig;
use deadpool_postgres::{Config, CreatePoolError, ManagerConfig, RecyclingMethod, Runtime};
use refinery::embed_migrations;
use serde::Deserialize;
use std::{ops::Deref, sync::Arc};
use tokio_postgres::NoTls;

embed_migrations!("../migrations");

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
        c.dbname = Some(config.database);
        c.manager = Some(ManagerConfig {
            recycling_method: RecyclingMethod::Verified,
        });
        c
    }
}

impl Database {
    pub fn new(config: DatabaseConfig) -> Result<Self, CreatePoolError> {
        Ok(Self(Arc::new(
            Config::from(config).create_pool(Some(Runtime::Tokio1), NoTls)?,
        )))
    }
}
