use async_trait::async_trait;

use crate::database::Database;

/// Test environment definition.
///
/// Different environments usually stands for different start or deploy manner,
/// like standalone versus cluster etc. [`Environment`] is sort of [`Database`]
/// "factory" - it create different [`Database`]s with different environment
/// mode parameter.
///
/// Environments are distingushed via the mode name (see the signature of
/// [`Self::start`] and [`Self::stop`]). And the mode name are got from the
/// directory name. Refer to crate level documentation for more information
/// about directory organizaiton rules.
#[async_trait]
pub trait Environment {
    type DB: Database;

    /// Start a [`Database`] to run test queries.
    ///
    /// Two parameters are the mode of this environment, or environment's name.
    /// And the config file's path to this environment if it's find, it's defined
    /// by the `env_config_file` field in the root config toml, and the default
    /// value is `config.toml`.
    async fn start(&self, env: &str, config: Option<String>) -> Self::DB;

    /// Stop one [`Database`].
    async fn stop(&self, env: &str, database: Self::DB);
}
