// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

#![allow(clippy::print_stdout)]

use std::{fmt::Display, path::Path};

use async_trait::async_trait;
use sqlness::{ConfigBuilder, Database, EnvController, Runner};

struct MyController;
struct MyDB;

#[async_trait]
impl Database for MyDB {
    async fn query(&mut self, _query: String) -> Box<dyn Display> {
        // Implement query logic here
        // println!("Exec {}...", query);
        return Box::new("ok".to_string());
    }
}

impl MyDB {
    fn new(_env: &str, _config: Option<&Path>) -> Self {
        MyDB
    }

    fn stop(self) {
        println!("MyDB stopped.");
    }
}

#[async_trait]
impl EnvController for MyController {
    type DB = MyDB;

    async fn start(&self, env: &str, config: Option<&Path>) -> Self::DB {
        println!("Start, env:{env}, config:{config:?}.");
        MyDB::new(env, config)
    }

    async fn stop(&self, env: &str, database: Self::DB) {
        println!("Stop, env:{env}.",);
        database.stop();
    }
}

#[tokio::main]
async fn main() {
    let env = MyController;
    let config = ConfigBuilder::default()
        .case_dir("examples/basic-case".to_string())
        .build()
        .unwrap();
    let runner = Runner::new_with_config(config, env)
        .await
        .expect("Create Runner failed");

    println!("Run testcase...");

    runner.run().await.unwrap();
}
