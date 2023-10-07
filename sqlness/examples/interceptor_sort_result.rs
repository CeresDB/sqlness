// Copyright 2023 CeresDB Project Authors. Licensed under Apache-2.0.

//! Shows how an SORT_RESULT interceptor works.

use std::{fmt::Display, path::Path};

use async_trait::async_trait;
use sqlness::{ConfigBuilder, Database, EnvController, QueryContext, Runner};

struct MyController;
struct MyDB;

#[async_trait]
impl Database for MyDB {
    async fn query(&self, _: QueryContext, query: String) -> Box<dyn Display> {
        return Box::new(query);
    }
}

impl MyDB {
    fn new(_env: &str, _config: Option<&Path>) -> Self {
        MyDB
    }

    fn stop(self) {}
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
        .case_dir("examples/interceptor-sort-result".to_string())
        .build()
        .unwrap();
    let runner = Runner::new(config, env);

    println!("Run testcase...");

    runner.run().await.unwrap();
}
