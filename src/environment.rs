use async_trait::async_trait;

use crate::database::Database;

#[async_trait]
pub trait Environment {
    type DB: Database;

    async fn start(&self, mode: &str, config: Option<String>) -> Self::DB;
    async fn stop(&self, mode: &str, database: Self::DB);
}
