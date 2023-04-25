// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

use regex::Regex;

use crate::interceptor::{Interceptor, InterceptorFactory, InterceptorRef};

const PREFIX: &str = "REPLACE";

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

impl Interceptor for ReplaceInterceptor {
    fn after_execute(&self, result: &mut String) {
        let re = Regex::new(&self.pattern).unwrap();
        let replaced = re.replace_all(result, &self.replacement);
        *result = replaced.to_string();
    }
}

pub struct ReplaceInterceptorFactory;

impl InterceptorFactory for ReplaceInterceptorFactory {
    fn try_new(&self, interceptor: &str) -> Option<InterceptorRef> {
        if interceptor.starts_with(PREFIX) {
            let args = interceptor
                .trim_start_matches(PREFIX)
                .trim_start()
                .trim_end();
            let mut args = args.splitn(2, ' ');
            let pattern = args.next()?.to_string();
            if pattern.is_empty() {
                return None;
            }
            let replacement = args.next().unwrap_or("").to_string();
            Some(Box::new(ReplaceInterceptor {
                pattern,
                replacement,
            }))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_replace_with_empty_string() {
        let input = "REPLACE ";
        let interceptor = ReplaceInterceptorFactory {}.try_new(input);
        assert!(interceptor.is_none());
    }

    #[test]
    fn replace_without_replacement() {
        let input = "REPLACE 0";
        let interceptor = ReplaceInterceptorFactory {}.try_new(input).unwrap();

        let mut exec_result = "000010101".to_string();
        interceptor.after_execute(&mut exec_result);
        assert_eq!(exec_result, "111".to_string());
    }

    #[test]
    fn simple_replace() {
        let input = "REPLACE 00 2";
        let interceptor = ReplaceInterceptorFactory {}.try_new(input).unwrap();

        let mut exec_result = "0000010101".to_string();
        interceptor.after_execute(&mut exec_result);
        assert_eq!(exec_result, "22010101".to_string());
    }
}
