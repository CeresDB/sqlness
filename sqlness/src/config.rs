// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

use derive_builder::Builder;

use crate::interceptor::{builtin_interceptors, InterceptorFactoryRef};

/// Configurations of [`Runner`].
///
/// [`Runner`]: crate::Runner
#[derive(Builder)]
pub struct Config {
    pub case_dir: String,
    /// Default value: `sql`
    #[builder(default = "Config::default_test_case_extension()")]
    pub test_case_extension: String,
    /// Default value: `result`
    #[builder(default = "Config::default_result_extension()")]
    pub result_extension: String,
    /// Default value: `-- SQLNESS`
    #[builder(default = "Config::default_interceptor_prefix()")]
    pub interceptor_prefix: String,
    /// Default value: `config.toml`
    #[builder(default = "Config::default_env_config_file()")]
    pub env_config_file: String,
    /// Fail this run as soon as one case fails if true
    #[builder(default = "Config::default_fail_fast()")]
    pub fail_fast: bool,
    /// Test only matched testcases, default `.*`
    /// Env is prepended before filename, eg `{env}:{filename}`
    #[builder(default = "Config::default_test_filter()")]
    pub test_filter: String,
    /// Test only matched env, default `.*`
    #[builder(default = "Config::default_env_filter()")]
    pub env_filter: String,
    /// Whether follow symbolic links when searching test case files.
    /// Defaults to "true" (follow symbolic links).
    #[builder(default = "Config::default_follow_links()")]
    pub follow_links: bool,

    /// Interceptors used to pre-process input query and post-process query response
    #[builder(default = "Config::default_interceptors()")]
    pub interceptor_factories: Vec<InterceptorFactoryRef>,
}

impl Config {
    fn default_test_case_extension() -> String {
        "sql".to_string()
    }

    fn default_result_extension() -> String {
        "result".to_string()
    }

    fn default_interceptor_prefix() -> String {
        "-- SQLNESS".to_string()
    }

    fn default_env_config_file() -> String {
        "config.toml".to_string()
    }

    fn default_fail_fast() -> bool {
        true
    }

    fn default_test_filter() -> String {
        ".*".to_string()
    }

    fn default_env_filter() -> String {
        ".*".to_string()
    }

    fn default_follow_links() -> bool {
        true
    }

    fn default_interceptors() -> Vec<InterceptorFactoryRef> {
        builtin_interceptors()
    }
}

/// Config for DatabaseBuilder
#[derive(Debug, Builder, Clone)]
pub struct DatabaseConfig {
    pub ip_or_host: String,
    pub tcp_port: u16,
    pub user: Option<String>,
    pub pass: Option<String>,
    pub db_name: Option<String>,
}
