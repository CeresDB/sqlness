// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

use std::{collections::HashMap, fmt::Display, path::Path};

use tokio::{
    fs::File,
    io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader},
};

use crate::{
    config::Config,
    error::Result,
    interceptor::{InterceptorFactoryRef, InterceptorRef},
    Database, SqlnessError,
};

const COMMENT_PREFIX: &str = "--";

pub(crate) struct TestCase {
    name: String,
    queries: Vec<Query>,
}

impl TestCase {
    pub(crate) async fn from_file<P: AsRef<Path>>(
        path: P,
        cfg: &Config,
        interceptor_factories: Vec<InterceptorFactoryRef>,
    ) -> Result<Self> {
        let file = File::open(path.as_ref())
            .await
            .map_err(|e| SqlnessError::ReadPath {
                source: e,
                path: path.as_ref().to_path_buf(),
            })?;

        let mut queries = vec![];
        let mut query = Query::with_interceptor_factories(interceptor_factories.clone());

        let mut lines = BufReader::new(file).lines();
        while let Some(line) = lines.next_line().await? {
            // intercept command start with INTERCEPTOR_PREFIX
            if line.starts_with(&cfg.interceptor_prefix) {
                query.push_interceptor(&cfg.interceptor_prefix, line);
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
                query = Query::with_interceptor_factories(interceptor_factories.clone());
            } else {
                query.append_query_line("\n");
            }
        }

        Ok(Self {
            name: path.as_ref().to_str().unwrap().to_string(),
            queries,
        })
    }

    pub(crate) async fn execute<W>(&mut self, db: &dyn Database, writer: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        for query in &mut self.queries {
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

/// A String-to-String map used as query context.
#[derive(Default, Debug)]
pub struct QueryContext {
    pub context: HashMap<String, String>,
}

#[derive(Default)]
struct Query {
    query_lines: Vec<String>,
    interceptor_lines: Vec<String>,
    interceptor_factories: Vec<InterceptorFactoryRef>,
    interceptors: Vec<InterceptorRef>,
}

impl Query {
    pub fn with_interceptor_factories(interceptor_factories: Vec<InterceptorFactoryRef>) -> Self {
        Self {
            interceptor_factories,
            ..Default::default()
        }
    }

    fn push_interceptor(&mut self, interceptor_prefix: &str, interceptor_line: String) {
        let interceptor_text = interceptor_line
            .trim_start_matches(interceptor_prefix)
            .trim_start();
        for factories in &self.interceptor_factories {
            if let Some(interceptor) = factories.try_new(interceptor_text) {
                self.interceptors.push(interceptor);
            }
        }
        self.interceptor_lines.push(interceptor_line);
    }

    fn append_query_line(&mut self, line: &str) {
        self.query_lines.push(line.to_string());
    }

    async fn execute<W>(&mut self, db: &dyn Database, writer: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        let context = self.before_execute_intercept();

        let mut result = db
            .query(context, self.concat_query_lines())
            .await
            .to_string();

        self.after_execute_intercept(&mut result);
        self.write_result(writer, result.to_string()).await?;

        Ok(())
    }

    fn before_execute_intercept(&mut self) -> QueryContext {
        let mut context = QueryContext::default();

        for interceptor in &self.interceptors {
            interceptor.before_execute(&mut self.query_lines, &mut context);
        }

        context
    }

    fn after_execute_intercept(&mut self, result: &mut String) {
        for interceptor in &self.interceptors {
            interceptor.after_execute(result);
        }
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
        for interceptor in &self.interceptor_lines {
            writer.write_all(interceptor.as_bytes()).await?;
            writer.write("\n".as_bytes()).await?;
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
