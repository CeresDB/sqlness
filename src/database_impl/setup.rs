// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

use std::{
    borrow::BorrowMut,
    cell::RefCell,
    fmt::Display,
    rc::Rc,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use mysql::{prelude::Queryable, Conn, OptsBuilder};

use crate::{config::DatabaseConnConfig, error::Result, Database, SqlnessError};

#[async_trait]
pub trait DatabaseBuilder: Send + Sync + Default {
    async fn build(&self, config: DatabaseConnConfig) -> Result<Box<dyn Database>>;
}

#[derive(Default)]
pub struct MysqlDatabaseBuilder;

#[derive(Debug)]
pub struct MysqlDatabase {
    conn: Arc<Mutex<Conn>>,
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
                for row in result {
                    match row {
                        Ok(row) => {
                            println!("{:?}", row);
                        }
                        Err(e) => {
                            todo!()
                        }
                    }
                }
                todo!()
            }
            Err(e) => format!("Failed to execute query, err: {:?}", e),
        })
    }
}

pub async fn build_default_database<T>(config: DatabaseConnConfig) -> Result<Box<dyn Database>>
where
    T: DatabaseBuilder,
{
    let builder = T::default();
    builder.build(config).await
}

#[cfg(test)]
mod tests {
    use std::sync::{Mutex, Arc};

    use mysql::{OptsBuilder, Conn};

    use crate::{
        config::DatabaseConnConfig,
        database_impl::{setup::{DatabaseBuilder, MysqlDatabase}, MysqlDatabaseBuilder}, SqlnessError,
    };

    macro_rules! aw {
        ($e:expr) => {
            tokio_test::block_on($e)
        };
    }

    #[test]
    fn test_db() {
        let config = DatabaseConnConfig {
            ip_or_host: "100.100.30.53".to_string(),
            tcp_port: 3306,
            user: "root".to_string(),
            pass: Some("123456".to_string()),
            db_name: "test".to_string(),
        };
        let opts = OptsBuilder::new()
            .ip_or_hostname(Some(config.ip_or_host.clone()))
            .tcp_port(config.tcp_port)
            .user(Some(config.user.clone()))
            .pass(config.pass.clone())
            .db_name(Some(config.db_name.clone()));

        let conn = Conn::new(opts).map_err(|e| SqlnessError::ConnFailed { source: e, config }).unwrap();
        let db = MysqlDatabase {
            conn: Arc::new(Mutex::new(conn)),
        };
        let query = "select * from hello";
        let result = MysqlDatabase::execute(query, Arc::clone(&db.conn));
        aw!(result);
        println!("ok");
    }
}
