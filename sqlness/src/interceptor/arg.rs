// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

use crate::case::QueryContext;
use crate::interceptor::{Interceptor, InterceptorFactory, InterceptorRef};

const PREFIX: &str = "ARG";

/// Pass arguments to the [QueryContext].
///
/// # Example
/// ``` sql
/// -- SQLNESS ARG arg1=value1 arg2=value2
/// SELECT * FROM t;
/// ```
///
/// # Format
/// The arguments are in the format of `key=value`, without space. The key should not contains
/// equal marker (`=`), and value can be any string without space. The arguments are separated
/// by spaces.
///
/// It will overwrite existing key-value pair in context if the key name is same.
#[derive(Debug)]
pub struct ArgInterceptor {
    args: Vec<(String, String)>,
}

impl Interceptor for ArgInterceptor {
    fn before_execute(&self, _: &mut Vec<String>, context: &mut QueryContext) {
        for (key, value) in &self.args {
            context.context.insert(key.to_string(), value.to_string());
        }
    }
}

pub struct ArgInterceptorFactory;

impl InterceptorFactory for ArgInterceptorFactory {
    fn try_new(&self, interceptor: &str) -> Option<InterceptorRef> {
        if interceptor.starts_with(PREFIX) {
            let args =
                Self::separate_key_value_pairs(interceptor.trim_start_matches(PREFIX).trim_start());
            Some(Box::new(ArgInterceptor { args }))
        } else {
            None
        }
    }
}

impl ArgInterceptorFactory {
    fn separate_key_value_pairs(input: &str) -> Vec<(String, String)> {
        let mut result = Vec::new();
        for pair in input.split(' ') {
            if let Some((key, value)) = pair.split_once('=') {
                result.push((key.to_string(), value.to_string()));
            }
        }
        result
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cut_arg_string() {
        let input = "ARG arg1=value1 arg2=value2 arg3=a=b=c arg4= arg5=,,,";
        let expected = vec![
            ("arg1".to_string(), "value1".to_string()),
            ("arg2".to_string(), "value2".to_string()),
            ("arg3".to_string(), "a=b=c".to_string()),
            ("arg4".to_string(), "".to_string()),
            ("arg5".to_string(), ",,,".to_string()),
        ];

        let args = ArgInterceptorFactory::separate_key_value_pairs(input);
        assert_eq!(args, expected);
    }
}
