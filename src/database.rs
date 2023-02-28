// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

use std::fmt::Display;

use crate::config::DatabaseConnConfig;
use async_trait::async_trait;

use crate::case::QueryContext;

/// Query executor.
///
/// [`Runner`] will call [`EnvController::start`] to create database to
/// execute query. The context parameter is a key-value pair map that
/// usually comes from interceptor or config file.
///
/// [`Runner`]: crate::Runner
/// [`EnvController::start`]: crate::EnvController#tymethod.start
#[async_trait]
pub trait Database {
    async fn query(&self, context: QueryContext, query: String) -> Box<dyn Display>;
}

#[async_trait]
pub trait DatabaseBuilder {
    type DB: Database;
    type Err;

    async fn build(&self, config: DatabaseConnConfig) -> Result<Self::DB, Self::Err>;
}
