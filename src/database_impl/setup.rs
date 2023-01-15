// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

use std::{sync::Arc, fmt::Display};

use async_trait::async_trait;
use mysql::{Conn, OptsBuilder, prelude::Queryable};

use crate::{config::DatabaseConnConfig, error::Result, Database, SqlnessError};

#[async_trait]
pub trait DatabaseBuilder: Send + Sync + Default {
    async fn build(&self, config: DatabaseConnConfig) -> Result<Arc<dyn Database>>;
}

#[derive(Default)]
pub struct MysqlDatabaseBuilder;

pub struct MysqlDatabase {
    conn: Conn
}

#[async_trait]
impl DatabaseBuilder for MysqlDatabaseBuilder {
    async fn build(&self, config: DatabaseConnConfig) -> Result<Arc<dyn Database>> {
        let opts = OptsBuilder::new()
            .ip_or_hostname(Some(config.ip_or_host.clone()))
            .tcp_port(config.tcp_port)
            .user(Some(config.user.clone()))
            .pass(config.pass.clone())
            .db_name(Some(config.db_name.clone()));

        let conn = Conn::new(opts).map_err(|e| SqlnessError::ConnFailed {
            source: e,
            config
        })?;

        todo!()
    }
}

#[async_trait]
impl Database for MysqlDatabase {

    async fn query(&self, query: String) -> Box<dyn Display> {
        // self.conn.query(query).map_err(|e| SqlnessError::QueryFailed { source: (), query: () })
        todo!()
    }

}

pub async fn build_default_database<T>(config: DatabaseConnConfig) -> Result<Arc<dyn Database>>
where
    T: DatabaseBuilder,
{
    let builder = T::default();
    builder.build(config).await
}

#[cfg(test)]
mod tests {}
