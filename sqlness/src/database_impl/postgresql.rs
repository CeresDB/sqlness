// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

use async_trait::async_trait;
use postgres::{Client, Config, NoTls, Row};
use std::{
    fmt::Display,
    sync::{Arc, Mutex},
};

use crate::{Database, DatabaseConfig, QueryContext};

pub struct PostgresqlDatabase {
    client: Arc<Mutex<Client>>,
}

impl PostgresqlDatabase {
    pub fn new(config: &DatabaseConfig) -> Result<Self, postgres::Error> {
        let mut postgres_config = Config::new();
        postgres_config
            .port(config.tcp_port)
            .host(&config.ip_or_host);

        if let Some(user) = &config.user {
            postgres_config.user(user);
        }
        if let Some(password) = &config.pass {
            postgres_config.password(password);
        }
        if let Some(dbname) = &config.db_name {
            postgres_config.dbname(dbname);
        }

        let client = postgres_config.connect(NoTls)?;
        Ok(PostgresqlDatabase {
            client: Arc::new(Mutex::new(client)),
        })
    }

    pub fn execute(query: &str, client: Arc<Mutex<Client>>) -> Box<dyn Display> {
        let mut client = match client.lock() {
            Ok(client) => client,
            Err(err) => {
                return Box::new(format!("Failed to get connection, encountered: {:?}", err))
            }
        };

        let result = match client.query(query, &[]) {
            Ok(rows) => {
                format!("{}", PostgresqlFormatter { rows })
            }
            Err(err) => format!("Failed to execute query, encountered: {:?}", err),
        };

        Box::new(result)
    }
}

struct PostgresqlFormatter {
    pub rows: Vec<Row>,
}

impl Display for PostgresqlFormatter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.rows.is_empty() {
            return f.write_fmt(format_args!("(Empty response)"));
        }

        let top = &self.rows[0];
        let columns = top
            .columns()
            .iter()
            .map(|column| column.name())
            .collect::<Vec<_>>();
        for col in &columns {
            f.write_fmt(format_args!("{},", col))?;
        }

        f.write_str("\n")?;

        for row in &self.rows {
            f.write_fmt(format_args!("{:?}\n", row))?;
        }

        Ok(())
    }
}

#[async_trait]
impl Database for PostgresqlDatabase {
    async fn query(&self, _: QueryContext, query: String) -> Box<dyn Display> {
        Self::execute(&query, Arc::clone(&self.client))
    }
}
