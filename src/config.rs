use serde::{Deserialize, Serialize};

/// The root configurations.
///
/// This struct is not intended to be used directly. It is deserialized from the
/// toml file passed to [`Runner::new`].
///
/// [`Runner::new`]: crate::Runner#method.new
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub case_dir: String,
    /// Default value: `.sql`
    pub test_case_extension: String,
    /// Default value: `.output`
    pub output_result_extension: String,
    /// Default value: `.result`
    pub expect_result_extension: String,
    /// Default value: `-- SQLNESS`
    pub interceptor_prefix: String,
    /// Default value: `config.toml`
    pub env_config_file: String,
}
