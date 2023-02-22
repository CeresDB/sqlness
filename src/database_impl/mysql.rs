// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

/// DatabaseBuilder for MySQL.
use std::{
    borrow::{Borrow, Cow},
    fmt::Display,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use mysql::{prelude::Queryable, Conn, OptsBuilder, Row};

use crate::{config::DatabaseConnConfig, database::DatabaseBuilder, Database};

#[derive(Default)]
pub struct MysqlBuilder;

/// How to test:
/// ```rust, ignore, no_run
///  fn test_db() {
///      let config = DatabaseConnConfig {
///          ip_or_host: "localhost".to_string(),
///          tcp_port: 3306,
///          user: "root".to_string(),
///          pass: Some("123456".to_string()),
///          db_name: "hello".to_string(),
///      };

///      let opts = OptsBuilder::new()
///          .ip_or_hostname(Some(config.ip_or_host.clone()))
///          .tcp_port(config.tcp_port)
///          .user(Some(config.user.clone()))
///          .pass(config.pass.clone())
///          .db_name(Some(config.db_name.clone()));
///      let conn = Conn::new(opts).unwrap();
///      let db = MysqlDatabase {
///          conn: Arc::new(Mutex::new(conn)),
///      };
///      let query = "select * from test;";
///      let future = MysqlDatabase::execute(query, Arc::clone(&db.conn));
///      let result = tokio_test::block_on(future);
///      println!("{}", result);
///  }
/// ```
#[derive(Debug)]
pub struct MysqlDatabase {
    conn: Arc<Mutex<Conn>>,
}

struct MysqlFormatter {
    pub rows: Vec<Row>,
}

#[async_trait]
impl DatabaseBuilder for MysqlBuilder {
    type DB = MysqlDatabase;
    type Err = mysql::Error;

    async fn build(&self, config: DatabaseConnConfig) -> Result<Self::DB, Self::Err> {
        let opts = OptsBuilder::new()
            .ip_or_hostname(Some(config.ip_or_host.clone()))
            .tcp_port(config.tcp_port)
            .user(Some(config.user.clone()))
            .pass(config.pass.clone())
            .db_name(Some(config.db_name));

        let conn = Conn::new(opts)?;
        Ok(MysqlDatabase {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}

#[async_trait]
impl Database for MysqlDatabase {
    async fn query(&self, query: String) -> Box<dyn Display> {
        Self::execute(&query, Arc::clone(&self.conn)).await
    }
}

impl MysqlDatabase {
    async fn execute(query: &str, connect: Arc<Mutex<Conn>>) -> Box<dyn Display> {
        let mut conn = match connect.lock() {
            Ok(conn) => conn,
            Err(e) => return Box::new(format!("Failed to get connection, err: {:?}", e)),
        };

        let result = conn.query_iter(query);
        Box::new(match result {
            Ok(result) => {
                let mut rows = vec![];
                let affected_rows = result.affected_rows();
                for row in result {
                    match row {
                        Ok(r) => rows.push(r),
                        Err(e) => {
                            return Box::new(format!("Failed to parse query result, err: {:?}", e))
                        }
                    }
                }

                if rows.is_empty() {
                    format!("affected_rows: {}", affected_rows)
                } else {
                    format!("{}", MysqlFormatter { rows })
                }
            }
            Err(e) => format!("Failed to execute query, err: {:?}", e),
        })
    }
}

impl Display for MysqlFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let head_column = &self.rows[0];
        let head_binding = head_column.columns();
        let names = head_binding
            .iter()
            .map(|column| column.name_str())
            .collect::<Vec<Cow<str>>>();
        for name in &names {
            f.write_fmt(format_args!("{},", name))?;
        }
        f.write_str("\n")?;

        for row in &self.rows {
            for column_name in &names {
                let name = column_name.borrow();
                f.write_fmt(format_args!("{:?},", row.get::<String, &str>(name)))?;
            }
            f.write_str("\n")?;
        }

        Ok(())
    }
}
