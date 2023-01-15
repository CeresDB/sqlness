// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

use std::path::PathBuf;

use thiserror::Error;

use crate::config::DatabaseConnConfig;

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

    #[error("Cannot parse the output/result file. Not valid UTF-8 encoding")]
    ReadResult(#[from] std::string::FromUtf8Error),

    #[error("Run failed. {count} cases can't pass")]
    RunFailed { count: usize },

    #[error("Failed to connection database server. config: {config}, error: {source}")]
    ConnFailed {
        source: mysql::Error,
        config: DatabaseConnConfig,
    },

    #[error("Failed to execute the sql statement. query: {query}, error: {source}")]
    QueryFailed {
        source: mysql::Error,
        query: String
    }
}

pub(crate) type Result<T> = std::result::Result<T, SqlnessError>;
