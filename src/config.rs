use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub(crate) case_dir: String,
    pub(crate) test_case_extension: String,
    pub(crate) output_result_extension: String,
    pub(crate) expect_result_extension: String,
    pub(crate) interceptor_prefix: String,
}
