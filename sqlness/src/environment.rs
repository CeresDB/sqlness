// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

use std::path::Path;

use async_trait::async_trait;

use crate::database::Database;

/// Controller of test environments.
///
/// Environments usually stand for different start or deploy manner,
/// like standalone versus cluster etc. [`EnvController`] is sort of [`Database`]
/// "factory" - it create different [`Database`]s with different environment
/// mode parameter.
///
/// Environments are distingushed via the mode name (see the signature of
/// [`Self::start`] and [`Self::stop`]). Those names are extracted from the first-level
/// directories of test case directory. Refer to crate level documentation for more information
/// about directory organizaiton rules.
#[async_trait]
pub trait EnvController: Send + Sync {
    type DB: Database;

    /// Start a [`Database`] to run test queries.
    ///
    /// Three parameters are the mode of this environment, or environment's name,
    /// the id of this database instance, and the config file's path to this environment if it's find,
    /// it's defined by the `env_config_file` field in the root config toml, and the default
    /// value is `config.toml`.
    ///
    /// The id is used to distinguish different database instances in the same environment.
    /// For example, you may want to run the sqlness test in parallel against different instances
    /// of the same environment to accelerate the test.
    async fn start(&self, env: &str, id: usize, config: Option<&Path>) -> Self::DB;

    /// Stop one [`Database`].
    async fn stop(&self, env: &str, database: Self::DB);
}
