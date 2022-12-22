// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

use derive_builder::Builder;
use serde::{Deserialize, Serialize};

/// Configurations of [`Runner`].
///
/// [`Runner`]: crate::Runner
#[derive(Debug, Serialize, Deserialize, Builder)]
pub struct Config {
    pub case_dir: String,
    /// Default value: `sql`
    #[builder(default = "String::from(\"sql\")")]
    pub test_case_extension: String,
    /// Default value: `output`
    #[builder(default = "String::from(\"output\")")]
    pub output_result_extension: String,
    /// Default value: `result`
    #[builder(default = "String::from(\"result\")")]
    pub expect_result_extension: String,
    /// Default value: `-- SQLNESS`
    #[builder(default = "String::from(\"-- SQLNESS\")")]
    pub interceptor_prefix: String,
    /// Default value: `config.toml`
    #[builder(default = "String::from(\"config.toml\")")]
    pub env_config_file: String,
    /// Fail this run as soon as one case fails if true
    #[builder(default = "true")]
    pub fail_fast: bool,
    /// If specified, only run cases containing this string in their names.
    #[builder(default = "String::new()")]
    pub test_filter: String,
}
