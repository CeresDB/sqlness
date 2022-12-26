use std::{env, fmt::Display, process};

use async_trait::async_trait;
use sqlness::{Database, Environment, Runner, SqlnessError};

struct MyEnv;
struct BasicDB;

#[async_trait]
impl Database for BasicDB {
    async fn query(&self, query: String) -> Box<dyn Display> {
        println!("Exec {}...", query);

        return Box::new("ok".to_string());
    }
}

impl BasicDB {
    fn new(_env: &str, _config: Option<String>) -> Self {
        BasicDB
    }

    fn stop(self: Self) {
        println!("BasicDB stopped...");
    }
}

#[async_trait]
impl Environment for MyEnv {
    type DB = BasicDB;

    async fn start(&self, env: &str, config: Option<String>) -> Self::DB {
        println!("MyEnv start, env:{}, config:{:?}", env, config);
        BasicDB::new(env, config)
    }

    async fn stop(&self, env: &str, database: Self::DB) {
        println!("MyEnv stop, env:{}", env,);
        database.stop();
    }
}

#[tokio::main]
async fn main() -> Result<(), SqlnessError> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} config-path", args[0]);
        process::exit(1);
    }

    let env = MyEnv;
    let config_path = &args[1];
    let runner = Runner::new(config_path, env)
        .await
        .expect("Create Runner failed");

    println!("get runner");

    runner.run().await?;

    Ok(())
}
