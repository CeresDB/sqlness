// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

use std::{
    borrow::{Borrow, Cow},
    fmt::Display,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use mysql::{prelude::Queryable, Conn, OptsBuilder, Row};

use crate::{config::DatabaseConnConfig, error::Result, Database, SqlnessError};

#[async_trait]
pub trait DatabaseBuilder: Send + Sync + Default {
    async fn build(&self, config: DatabaseConnConfig) -> Result<Box<dyn Database>>;
}

#[derive(Default)]
pub struct MysqlDatabaseBuilder;

/// How to test:
/// ```rust, ignore, no_run
///  fn test_db() {
///      let config = DatabaseConnConfig {
///          ip_or_host: "localhost".to_string(),
///          tcp_port: 3306,
///          user: "root".to_string(),
///          pass: Some("123456".to_string()),
///          db_name: "hellp".to_string(),
///      };

///      let opts = OptsBuilder::new()
///          .ip_or_hostname(Some(config.ip_or_host.clone()))
///          .tcp_port(config.tcp_port)
///          .user(Some(config.user.clone()))
///          .pass(config.pass.clone())
///          .db_name(Some(config.db_name.clone()));
///      let conn = Conn::new(opts)
///          .map_err(|e| SqlnessError::ConnFailed { source: e, config })
///          .unwrap();
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
impl DatabaseBuilder for MysqlDatabaseBuilder {
    async fn build(&self, config: DatabaseConnConfig) -> Result<Box<dyn Database>> {
        let opts = OptsBuilder::new()
            .ip_or_hostname(Some(config.ip_or_host.clone()))
            .tcp_port(config.tcp_port)
            .user(Some(config.user.clone()))
            .pass(config.pass.clone())
            .db_name(Some(config.db_name.clone()));

        let conn = Conn::new(opts).map_err(|e| SqlnessError::ConnFailed { source: e, config })?;
        Ok(Box::new(MysqlDatabase {
            conn: Arc::new(Mutex::new(conn)),
        }))
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
            Err(e) => return Box::new(format!("Failed to get connect, err: {:?}", e)),
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
                            return Box::new(format!("Failed to parser query result, err: {:?}", e))
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

pub async fn _build_default_database<T>(config: DatabaseConnConfig) -> Result<Box<dyn Database>>
where
    T: DatabaseBuilder,
{
    let builder = T::default();
    builder.build(config).await
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
