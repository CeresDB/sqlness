// Copyright 2023 CeresDB Project Authors. Licensed under Apache-2.0.

use std::collections::HashMap;

use crate::case::QueryContext;
use crate::error::Result;
use crate::interceptor::{Interceptor, InterceptorFactory, InterceptorRef};

pub const PREFIX: &str = "ENV";

/// Read environment variables and fill them in query.
///
/// # Example
/// ``` sql
/// -- SQLNESS ENV SECRET
/// SELECT $SECRET;
/// ```
///
/// Environment variables declared in `ENV` interceptor will be replaced in the
/// going to be executed. It won't be rendered in the result file so you can
/// safely put secret things in your query.
///
/// Note that only decalred and present environment variables will be replaced.
///
/// You can either declare multiple env in one intercetor or separate them into
/// different interceptors. The following two examples are equivalent:
///
/// ``` sql
/// -- SQLNESS ENV SECRET1 SECRET2
/// SELECT $SECRET1, $SECRET2;
///
/// -- SQLNESS ENV SECRET1
/// -- SQLNESS ENV SECRET2
/// SELECT $SECRET1, $SECRET2;
/// ````
#[derive(Debug)]
pub struct EnvInterceptor {
    /// Environment variables to be replaced.
    data: HashMap<String, String>,
}

impl Interceptor for EnvInterceptor {
    fn before_execute(&self, execute_query: &mut Vec<String>, _: &mut QueryContext) {
        for line in execute_query {
            for (key, value) in &self.data {
                let rendered = line.replace(key, value);
                *line = rendered;
            }
        }
    }
}

pub struct EnvInterceptorFactory;

impl InterceptorFactory for EnvInterceptorFactory {
    fn try_new(&self, ctx: &str) -> Result<InterceptorRef> {
        Self::create(ctx).map(|v| Box::new(v) as _)
    }
}

impl EnvInterceptorFactory {
    fn create(s: &str) -> Result<EnvInterceptor> {
        let input = s.trim_start().trim_end();
        let envs = input.split(' ').collect::<Vec<_>>();

        let mut data = HashMap::new();
        for env in envs {
            if let Ok(value) = std::env::var(env) {
                data.insert(format!("${env}"), value);
            }
        }

        Ok(EnvInterceptor { data })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cut_env_string() {
        let input = "SECRET NONEXISTENT";
        std::env::set_var("SECRET", "2333");

        let expected = [("$SECRET".to_string(), "2333".to_string())]
            .into_iter()
            .collect();

        let interceptor = EnvInterceptorFactory::create(input).unwrap();
        assert_eq!(interceptor.data, expected);
    }
}
