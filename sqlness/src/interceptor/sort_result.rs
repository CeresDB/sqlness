// Copyright 2023 CeresDB Project Authors. Licensed under Apache-2.0.

use std::collections::VecDeque;

use crate::{
    error::Result,
    interceptor::{Interceptor, InterceptorFactory, InterceptorRef},
    SqlnessError,
};

pub const PREFIX: &str = "SORT_RESULT";

/// Sort the query result in lexicographical order.
///
/// Grammar:
/// ``` text
/// -- SQLNESS SORT_RESULT <ignore-head> <ignore-tail>
/// ```
///
/// Both `ignore-head` and `ignore-tail` are optional. Default value is 0 (no lines will be ignored).
///
/// # Example
/// `.sql` file:
/// ``` sql
/// -- SQLNESS SORT_RESULT
/// SELECT * from values (3), (2), (1);
/// ```
///
/// `.result` file:
/// ``` sql
/// -- SQLNESS SORT_RESULT
/// SELECT * from values (3), (2), (1);
///
/// 1
/// 2
/// 3
/// ```
#[derive(Debug)]
pub struct SortResultInterceptor {
    /// How much lines to ignore from the head
    ignore_head: usize,
    /// How much lines to ignore from the tail
    ignore_tail: usize,
}

#[async_trait::async_trait]
impl Interceptor for SortResultInterceptor {
    async fn after_execute(&self, result: &mut String) {
        let mut lines = result.lines().collect::<VecDeque<_>>();
        let mut head = Vec::with_capacity(self.ignore_head);
        let mut tail = Vec::with_capacity(self.ignore_tail);

        // ignore head and tail
        for _ in 0..self.ignore_head {
            if let Some(l) = lines.pop_front() {
                head.push(l);
            }
        }
        for _ in 0..self.ignore_tail {
            if let Some(l) = lines.pop_back() {
                tail.push(l);
            }
        }
        tail.reverse();

        // sort remaining lines
        lines.make_contiguous().sort();

        let new_lines = head
            .into_iter()
            .chain(lines)
            .chain(tail)
            .collect::<Vec<_>>();
        *result = new_lines.join("\n");
    }
}

pub struct SortResultInterceptorFactory;

impl InterceptorFactory for SortResultInterceptorFactory {
    fn try_new(&self, ctx: &str) -> Result<InterceptorRef> {
        let mut args = ctx.splitn(2, ' ').filter(|s| !s.is_empty());
        let ignore_head =
            args.next()
                .unwrap_or("0")
                .parse()
                .map_err(|e| SqlnessError::InvalidContext {
                    prefix: PREFIX.to_string(),
                    msg: format!("Expect number, err:{e}"),
                })?;
        let ignore_tail =
            args.next()
                .unwrap_or("0")
                .parse()
                .map_err(|e| SqlnessError::InvalidContext {
                    prefix: PREFIX.to_string(),
                    msg: format!("Expect number, err:{e}"),
                })?;

        Ok(Box::new(SortResultInterceptor {
            ignore_head,
            ignore_tail,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_with_negative() {
        let interceptor = SortResultInterceptorFactory.try_new("-1");
        assert!(interceptor.is_err());
    }

    #[tokio::test]
    async fn sort_result_full() {
        let interceptor = SortResultInterceptorFactory.try_new("").unwrap();

        let cases = [
            (
                String::from(
                    "abc\
                    \ncde\
                    \nefg",
                ),
                String::from(
                    "abc\
                    \ncde\
                    \nefg",
                ),
            ),
            (
                String::from(
                    "efg\
                    \ncde\
                    \nabc",
                ),
                String::from(
                    "abc\
                    \ncde\
                    \nefg",
                ),
            ),
        ];

        for (mut input, expected) in cases {
            interceptor.after_execute(&mut input).await;
            assert_eq!(input, expected);
        }
    }

    #[tokio::test]
    async fn ignore_head_exceeds_length() {
        let interceptor = SortResultInterceptorFactory.try_new("10000").unwrap();

        let mut exec_result = String::from(
            "3\
            \n2\
            \n1",
        );
        let expected = exec_result.clone();
        interceptor.after_execute(&mut exec_result).await;
        assert_eq!(exec_result, expected);
    }

    #[tokio::test]
    async fn ignore_tail_exceeds_length() {
        let interceptor = SortResultInterceptorFactory.try_new("0 10000").unwrap();

        let mut exec_result = String::from(
            "3\
            \n2\
            \n1",
        );
        let expected = exec_result.clone();
        interceptor.after_execute(&mut exec_result).await;
        assert_eq!(exec_result, expected);
    }
}
