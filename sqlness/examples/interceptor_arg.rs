// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

//! Shows how an ARG interceptor works.

use std::{fmt::Display, path::Path};

use async_trait::async_trait;
use sqlness::{builtin_interceptors, ConfigBuilder, Database, EnvController, QueryContext, Runner};

struct MyController;
struct MyDB;

#[async_trait]
impl Database for MyDB {
    async fn query(&self, ctx: QueryContext, _query: String) -> Box<dyn Display> {
        let mut args = ctx.context.into_iter().collect::<Vec<_>>();
        args.sort();
        let result = args
            .into_iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("\n");
        return Box::new(result);
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
        .case_dir("examples/interceptor-arg".to_string())
        .interceptor_factories(builtin_interceptors())
        .build()
        .unwrap();
    let runner = Runner::try_new(config, env)
        .await
        .expect("Create Runner failed");

    println!("Run testcase...");

    runner.run().await.unwrap();
}
