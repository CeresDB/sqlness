// Copyright 2023 CeresDB Project Authors. Licensed under Apache-2.0.

use std::path::Path;

use async_trait::async_trait;
use clap::Parser;
use futures::executor::block_on;
use sqlness::{
    database_impl::mysql::MysqlDatabase, ConfigBuilder, DatabaseConfig, DatabaseConfigBuilder,
    EnvController, Runner,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
/// A cli to run sqlness tests.
struct Args {
    /// Directory of test cases
    #[clap(short, long, required(true))]
    case_dir: String,

    /// IP of database to test against
    #[clap(short, long, required(true))]
    ip: String,

    /// Port of database to test against
    #[clap(short, long, required(true))]
    port: u16,

    /// User of database to test against
    #[clap(short, long, required(false))]
    user: Option<String>,

    /// Password of database to test against
    #[clap(short('P'), long, required(false))]
    password: Option<String>,

    /// DB name of database to test against
    #[clap(short, long)]
    db: Option<String>,

    /// Which DBMS to test against
    #[clap(short, long)]
    #[arg(value_enum, default_value_t)]
    r#type: DBType,
}

#[derive(clap::ValueEnum, Clone, Debug, Default)]
enum DBType {
    #[default]
    Mysql,
}

struct CliController {
    db_config: DatabaseConfig,
}

#[async_trait]
impl EnvController for CliController {
    type DB = MysqlDatabase;

    async fn start(&self, _env: &str, _config: Option<&Path>) -> Self::DB {
        MysqlDatabase::try_new(self.db_config.clone()).expect("build db")
    }

    async fn stop(&self, _env: &str, _db: Self::DB) {}
}

fn main() {
    println!("Begin run tests...");
    let args = Args::parse();
    let db_config = DatabaseConfigBuilder::default()
        .ip_or_host(args.ip)
        .tcp_port(args.port)
        .user(args.user)
        .pass(args.password)
        .db_name(args.db)
        .build()
        .expect("build db config");

    let ctrl = CliController { db_config };
    let config = ConfigBuilder::default()
        .case_dir(args.case_dir)
        .build()
        .expect("build config");

    block_on(async {
        let runner = Runner::new(config, ctrl);

        runner.run().await.expect("run testcase")
    });

    println!("Test run finish");
}
