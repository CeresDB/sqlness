// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

use std::{fmt::Display, path::Path};

use tokio::{
    fs::File,
    io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader},
};

use crate::{config::Config, error::Result, Database, SqlnessError};

const COMMENT_PREFIX: &str = "--";

pub(crate) struct TestCase {
    name: String,
    queries: Vec<Query>,
}

impl TestCase {
    pub(crate) async fn from_file<P: AsRef<Path>>(path: P, cfg: &Config) -> Result<Self> {
        let file = File::open(path.as_ref())
            .await
            .map_err(|e| SqlnessError::ReadPath {
                source: e,
                path: path.as_ref().to_path_buf(),
            })?;

        let mut queries = vec![];
        let mut query = Query::default();

        let mut lines = BufReader::new(file).lines();
        while let Some(line) = lines.next_line().await? {
            // intercept command start with INTERCEPTOR_PREFIX
            if line.starts_with(&cfg.interceptor_prefix) {
                query.push_interceptor(line);
                continue;
            }

            // ignore comment and empty line
            if line.starts_with(COMMENT_PREFIX) || line.is_empty() {
                continue;
            }

            query.append_query_line(&line);

            // SQL statement ends with ';'
            if line.ends_with(';') {
                queries.push(query);
                query = Query::default();
            } else {
                query.append_query_line("\n");
            }
        }

        Ok(Self {
            name: path.as_ref().to_str().unwrap().to_string(),
            queries,
        })
    }

    pub(crate) async fn execute<W>(&self, db: &dyn Database, writer: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        for query in &self.queries {
            query.execute(db, writer).await?;
        }

        Ok(())
    }
}

impl Display for TestCase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(Default)]
struct Query {
    query_lines: Vec<String>,
    interceptors: Vec<String>,
}

impl Query {
    fn push_interceptor(&mut self, post_process: String) {
        self.interceptors.push(post_process);
    }

    fn append_query_line(&mut self, line: &str) {
        self.query_lines.push(line.to_string());
    }

    async fn execute<W>(&self, db: &dyn Database, writer: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        let result = db.query(self.concat_query_lines()).await;
        self.write_result(writer, result.to_string()).await?;

        Ok(())
    }

    fn concat_query_lines(&self) -> String {
        self.query_lines
            .iter()
            .fold(String::new(), |query, str| query + " " + str)
    }

    #[allow(clippy::unused_io_amount)]
    async fn write_result<W>(&self, writer: &mut W, result: String) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        for interceptor in &self.interceptors {
            writer.write_all(interceptor.as_bytes()).await?;
        }
        for line in &self.query_lines {
            writer.write_all(line.as_bytes()).await?;
        }
        writer.write("\n\n".as_bytes()).await?;
        writer.write_all(result.as_bytes()).await?;
        writer.write("\n\n".as_bytes()).await?;

        Ok(())
    }
}
