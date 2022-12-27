// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

use std::fmt::Display;

use async_trait::async_trait;

/// Query executor.
///
/// [`Runner`] will call [`EnvController::start`] to create database to
/// execute query.
///
/// [`Runner`]: crate::Runner
/// [`EnvController::start`]: crate::EnvController#tymethod.start
#[async_trait]
pub trait Database {
    async fn query(&self, query: String) -> Box<dyn Display>;
}
