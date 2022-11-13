use std::fmt::Display;

use async_trait::async_trait;

#[async_trait]
pub trait Database {
    async fn query(&self, query: String) -> Box<dyn Display>;
}
