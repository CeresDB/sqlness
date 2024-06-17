// Copyright 2024 CeresDB Project Authors. Licensed under Apache-2.0.

use minijinja::Environment;
use serde_json::Value;

use crate::error::Result;
use crate::interceptor::{Interceptor, InterceptorFactory, InterceptorRef};
use crate::SqlnessError;

pub struct TemplateInterceptorFactory;

pub const PREFIX: &str = "TEMPLATE";

/// Templated query, powered by [minijinja](https://github.com/mitsuhiko/minijinja).
/// The template syntax can be found [here](https://docs.rs/minijinja/latest/minijinja/syntax/index.html).
///
/// Grammar:
/// ``` text
/// -- SQLNESS TEMPLATE <json>
/// ```
///
/// `json` define data bindings passed to template, it should be a valid JSON string.
///
/// # Example
/// `.sql` file:
/// ``` sql
/// -- SQLNESS TEMPLATE {"name": "test"}
/// SELECT * FROM table where name = "{{name}}"
/// ```
///
/// `.result` file:
/// ``` sql
/// -- SQLNESS TEMPLATE {"name": "test"}
/// SELECT * FROM table where name = "test";
/// ```
///
/// In order to generate multiple queries, you can use the builtin function
/// `sql_delimiter()` to insert a delimiter.
///
#[derive(Debug)]
pub struct TemplateInterceptor {
    data_bindings: Value,
}

fn sql_delimiter() -> std::result::Result<String, minijinja::Error> {
    Ok(";".to_string())
}

#[async_trait::async_trait]
impl Interceptor for TemplateInterceptor {
    async fn before_execute(
        &self,
        execute_query: &mut Vec<String>,
        _context: &mut crate::QueryContext,
    ) {
        let input = execute_query.join("\n");
        let mut env = Environment::new();
        env.add_function("sql_delimiter", sql_delimiter);
        env.add_template("sql", &input).unwrap();
        let tmpl = env.get_template("sql").unwrap();
        let rendered = tmpl.render(&self.data_bindings).unwrap();
        *execute_query = rendered
            .split('\n')
            .map(|v| v.to_string())
            .collect::<Vec<_>>();
    }
}

impl InterceptorFactory for TemplateInterceptorFactory {
    fn try_new(&self, ctx: &str) -> Result<InterceptorRef> {
        let data_bindings = if ctx.is_empty() {
            serde_json::from_str("{}")
        } else {
            serde_json::from_str(ctx)
        }
        .map_err(|e| SqlnessError::InvalidContext {
            prefix: PREFIX.to_string(),
            msg: format!("Expect json, err:{e}"),
        })?;

        Ok(Box::new(TemplateInterceptor { data_bindings }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn basic_template() {
        let interceptor = TemplateInterceptorFactory
            .try_new(r#"{"name": "test"}"#)
            .unwrap();

        let mut input = vec!["SELECT * FROM table where name = '{{name}}'".to_string()];
        interceptor
            .before_execute(&mut input, &mut crate::QueryContext::default())
            .await;

        assert_eq!(input, vec!["SELECT * FROM table where name = 'test'"]);
    }

    #[tokio::test]
    async fn vector_template() {
        let interceptor = TemplateInterceptorFactory
            .try_new(r#"{"aggr": ["sum", "count", "avg"]}"#)
            .unwrap();

        let mut input = [
            "{%- for item in aggr %}",
            "SELECT {{item}}(c) from t;",
            "{%- endfor %}",
        ]
        .map(|v| v.to_string())
        .to_vec();
        interceptor
            .before_execute(&mut input, &mut crate::QueryContext::default())
            .await;

        assert_eq!(
            input,
            [
                "",
                "SELECT sum(c) from t;",
                "SELECT count(c) from t;",
                "SELECT avg(c) from t;"
            ]
            .map(|v| v.to_string())
            .to_vec()
        );
    }

    #[tokio::test]
    async fn range_template() {
        let interceptor = TemplateInterceptorFactory.try_new(r#""#).unwrap();

        let mut input = [
            "INSERT INTO t (c) VALUES",
            "{%- for num in range(1, 5) %}",
            "({{ num }}){%if not loop.last %}, {% endif %}",
            "{%- endfor %}",
            ";",
        ]
        .map(|v| v.to_string())
        .to_vec();
        interceptor
            .before_execute(&mut input, &mut crate::QueryContext::default())
            .await;

        assert_eq!(
            input,
            [
                "INSERT INTO t (c) VALUES",
                "(1), ",
                "(2), ",
                "(3), ",
                "(4)",
                ";"
            ]
            .map(|v| v.to_string())
            .to_vec()
        );
    }
}
