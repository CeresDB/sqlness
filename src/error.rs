use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SqlnessError {
    #[error("Unable to read from path {path}")]
    ReadPath {
        source: std::io::Error,
        path: PathBuf,
    },

    #[error("Failed to parse toml file {file}, error: {source}")]
    ParseToml {
        source: toml::de::Error,
        file: PathBuf,
    },

    #[error("IO operation failed, source error: {0}")]
    IO(#[from] std::io::Error),
}

pub(crate) type Result<T> = std::result::Result<T, SqlnessError>;
