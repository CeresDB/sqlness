// Copyright 2023 CeresDB Project Authors. Licensed under Apache-2.0.

use crate::error::Result;
use crate::interceptor::{Interceptor, InterceptorFactory, InterceptorRef};
use crate::SqlnessError;
use regex::Regex;

pub const PREFIX: &str = "REPLACE";

/// Replace all matched occurrences in execution result to the given string.
/// The pattern is treated as a regular expression.
///
/// Grammar:
/// ``` text
/// -- SQLNESS REPLACE <pattern> <replacement>
/// ```
///
/// `replacement` is optional. If not specified, it will be replaced with an empty string.
///
/// # Example
/// `.sql` file:
/// ``` sql
/// -- SQLNESS REPLACE 0 1
/// SELECT 0;
/// ```
///
/// `.result` file:
/// ``` sql
/// -- SQLNESS REPLACE 0 1
/// SELECT 0;
///
/// 1
/// ```
///
/// Multiple `REPLACE` statements are allowed to one query. They will be evaluated in order.
#[derive(Debug)]
pub struct ReplaceInterceptor {
    pattern: String,
    replacement: String,
}

#[async_trait::async_trait]
impl Interceptor for ReplaceInterceptor {
    async fn after_execute(&self, result: &mut String) {
        let re = Regex::new(&self.pattern).unwrap();
        let replaced = re.replace_all(result, &self.replacement);
        *result = replaced.to_string();
    }
}

pub struct ReplaceInterceptorFactory;

impl InterceptorFactory for ReplaceInterceptorFactory {
    fn try_new(&self, ctx: &str) -> Result<InterceptorRef> {
        // TODO(ruihang): support pattern with blanks
        let mut args = ctx.splitn(2, ' ');
        let pattern = args
            .next()
            .ok_or_else(|| SqlnessError::InvalidContext {
                prefix: PREFIX.to_string(),
                msg: "Expect <pattern> [replacement]".to_string(),
            })?
            .to_string();
        if pattern.is_empty() {
            return Err(SqlnessError::InvalidContext {
                prefix: PREFIX.to_string(),
                msg: "Pattern shouldn't be empty".to_string(),
            });
        }
        let replacement = args.next().unwrap_or("").to_string();
        Ok(Box::new(ReplaceInterceptor {
            pattern,
            replacement,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_replace_with_empty_string() {
        let interceptor = ReplaceInterceptorFactory {}.try_new("");
        assert!(interceptor.is_err());
    }

    #[tokio::test]
    async fn replace_without_replacement() {
        let interceptor = ReplaceInterceptorFactory {}.try_new("0").unwrap();

        let mut exec_result = "000010101".to_string();
        interceptor.after_execute(&mut exec_result).await;
        assert_eq!(exec_result, "111".to_string());
    }

    #[tokio::test]
    async fn simple_replace() {
        let interceptor = ReplaceInterceptorFactory {}.try_new("00 2").unwrap();

        let mut exec_result = "0000010101".to_string();
        interceptor.after_execute(&mut exec_result).await;
        assert_eq!(exec_result, "22010101".to_string());
    }
}
