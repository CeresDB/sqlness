// Copyright 2023 CeresDB Project Authors. Licensed under Apache-2.0.

use std::{error::Error, fmt::Display, path::Path};

use async_trait::async_trait;
use clap::Parser;
use futures::executor::block_on;
use sqlness::{
    database_impl::{mysql::MysqlDatabase, postgresql::PostgresqlDatabase},
    ConfigBuilder, Database, DatabaseConfig, DatabaseConfigBuilder, EnvController, QueryContext,
    Runner,
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

#[derive(clap::ValueEnum, Clone, Debug, Default, Copy)]
enum DBType {
    #[default]
    Mysql,
    Postgresql,
}

struct DBProxy {
    database: Box<dyn Database + Sync + Send>,
}

#[async_trait]
impl Database for DBProxy {
    async fn query(&self, context: QueryContext, query: String) -> Box<dyn Display> {
        self.database.query(context, query).await
    }
}

impl DBProxy {
    pub fn try_new(db_config: DatabaseConfig, db_type: DBType) -> Result<Self, Box<dyn Error>> {
        let db = match db_type {
            DBType::Mysql => {
                Box::new(MysqlDatabase::try_new(db_config).expect("build db")) as Box<_>
            }
            DBType::Postgresql => {
                Box::new(PostgresqlDatabase::new(&db_config).expect("build db")) as Box<_>
            }
        };
        Ok(DBProxy { database: db })
    }
}

struct CliController {
    db_config: DatabaseConfig,
    db_type: DBType,
}

impl CliController {
    fn new(db_config: DatabaseConfig, db_type: DBType) -> Self {
        Self { db_config, db_type }
    }
}

#[async_trait]
impl EnvController for CliController {
    type DB = DBProxy;

    async fn start(&self, _env: &str, _config: Option<&Path>) -> Self::DB {
        DBProxy::try_new(self.db_config.clone(), self.db_type).expect("build db")
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

    let config = ConfigBuilder::default()
        .case_dir(args.case_dir)
        .build()
        .expect("build config");

    block_on(async {
        let cli = CliController::new(db_config, args.r#type);
        let runner = Runner::new(config, cli);
        runner.run().await.expect("run testcase")
    });

    println!("Test run finish");
}
