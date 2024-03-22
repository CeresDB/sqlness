// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

//! Shows how an ARG interceptor works.

use std::{fmt::Display, path::Path};

use async_trait::async_trait;
use sqlness::{ConfigBuilder, Database, EnvController, QueryContext, Runner};

struct MyController;
struct MyDB;

#[async_trait]
impl Database for MyDB {
    async fn query(&self, ctx: QueryContext, query: String) -> Box<dyn Display> {
        let mut args = ctx.context.into_iter().collect::<Vec<_>>();
        if args.is_empty() {
            return Box::new(query);
        }

        args.sort();
        let args = args
            .into_iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join(", ");
        let result = format!("Args: {args}\n\n{query}");
        Box::new(result)
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
    std::env::set_var("ENV1", "value1");
    std::env::set_var("ENV2", "value2");
    let env = MyController;
    let config = ConfigBuilder::default()
        .case_dir("examples/interceptor-case".to_string())
        .build()
        .unwrap();
    let runner = Runner::new(config, env);

    println!("Run testcase...");

    runner.run().await.unwrap();
}
