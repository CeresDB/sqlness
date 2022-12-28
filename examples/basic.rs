// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

use std::{env, fmt::Display, process};

use async_trait::async_trait;
use sqlness::{Database, EnvController, Runner};

struct MyController;
struct MyDB;

#[async_trait]
impl Database for MyDB {
    async fn query(&self, _query: String) -> Box<dyn Display> {
        // Implement query logic here
        // println!("Exec {}...", query);
        return Box::new("ok".to_string());
    }
}

impl MyDB {
    fn new(_env: &str, _config: Option<String>) -> Self {
        MyDB
    }

    fn stop(self) {
        println!("MyDB stopped.");
    }
}

#[async_trait]
impl EnvController for MyController {
    type DB = MyDB;

    async fn start(&self, env: &str, config: Option<String>) -> Self::DB {
        println!("Start, env:{}, config:{:?}.", env, config);
        MyDB::new(env, config)
    }

    async fn stop(&self, env: &str, database: Self::DB) {
        println!("Stop, env:{}.", env,);
        database.stop();
    }
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} config-path", args[0]);
        process::exit(1);
    }

    let env = MyController;
    let config_path = &args[1];
    let runner = Runner::try_new(config_path, env)
        .await
        .expect("Create Runner failed");

    println!("Run testcase...");

    runner.run().await.unwrap();
}
