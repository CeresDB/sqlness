// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

//! A demo designed to run failed.
//!
//! When there is any diff between old and new result,
//! Users must resolve the diff, and keep the result file up to date.

use std::{fmt::Display, fs::File, path::Path};

use async_trait::async_trait;
use sqlness::{ConfigBuilder, Database, EnvController, Runner};

struct MyController;
struct MyDB;

#[async_trait]
impl Database for MyDB {
    async fn query(&self, _query: String) -> Box<dyn Display> {
        return Box::new("Unexpected".to_string());
    }
}

// Used as a flag to indicate MyDB has started
const LOCK_FILE: &str = "/tmp/sqlness-bad-example.lock";

impl MyDB {
    fn new(_env: &str, _config: Option<&Path>) -> Self {
        File::create(LOCK_FILE).unwrap();
        MyDB
    }

    fn stop(self) {
        std::fs::remove_file(LOCK_FILE).unwrap();
    }
}

#[async_trait]
impl EnvController for MyController {
    type DB = MyDB;

    async fn start(&self, env: &str, config: Option<&Path>) -> Self::DB {
        MyDB::new(env, config)
    }

    async fn stop(&self, _env: &str, database: Self::DB) {
        database.stop();
    }
}

#[tokio::main]
async fn main() {
    let env = MyController;
    let config = ConfigBuilder::default()
        .case_dir("examples/bad-case".to_string())
        .build()
        .unwrap();
    let runner = Runner::new_with_config(config, env)
        .await
        .expect("Create Runner failed");

    println!("Run testcase...");

    runner.run().await.unwrap();
}
