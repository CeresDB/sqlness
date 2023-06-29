// Copyright 2023 CeresDB Project Authors. Licensed under Apache-2.0.

use std::collections::VecDeque;

use crate::interceptor::{Interceptor, InterceptorFactory, InterceptorRef};

const PREFIX: &str = "SORT_RESULT";

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

impl Interceptor for SortResultInterceptor {
    fn after_execute(&self, result: &mut String) {
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
            .chain(lines.into_iter())
            .chain(tail.into_iter())
            .collect::<Vec<_>>();
        *result = new_lines.join("\n");
    }
}

pub struct SortResultInterceptorFactory;

impl InterceptorFactory for SortResultInterceptorFactory {
    fn try_new(&self, interceptor: &str) -> Option<InterceptorRef> {
        Self::try_new_from_str(interceptor).map(|i| Box::new(i) as _)
    }
}

impl SortResultInterceptorFactory {
    fn try_new_from_str(interceptor: &str) -> Option<SortResultInterceptor> {
        if interceptor.starts_with(PREFIX) {
            let args = interceptor
                .trim_start_matches(PREFIX)
                .trim_start()
                .trim_end();
            let mut args = args.splitn(2, ' ').filter(|s| !s.is_empty());
            let ignore_head = args.next().unwrap_or("0").parse().ok()?;
            let ignore_tail = args.next().unwrap_or("0").parse().ok()?;

            Some(SortResultInterceptor {
                ignore_head,
                ignore_tail,
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn construct_with_empty_string() {
        let input = "SORT_RESULT";
        let sort_result = SortResultInterceptorFactory::try_new_from_str(input).unwrap();
        assert_eq!(sort_result.ignore_head, 0);
        assert_eq!(sort_result.ignore_tail, 0);
    }

    #[test]
    fn construct_with_negative() {
        let input = "SORT_RESULT -1";
        let interceptor = SortResultInterceptorFactory.try_new(input);
        assert!(interceptor.is_none());
    }

    #[test]
    fn sort_result_full() {
        let input = "SORT_RESULT";
        let interceptor = SortResultInterceptorFactory.try_new(input).unwrap();

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
            interceptor.after_execute(&mut input);
            assert_eq!(input, expected);
        }
    }

    #[test]
    fn ignore_head_exceeds_length() {
        let input = "SORT_RESULT 10000";
        let interceptor = SortResultInterceptorFactory.try_new(input).unwrap();

        let mut exec_result = String::from(
            "3\
            \n2\
            \n1",
        );
        let expected = exec_result.clone();
        interceptor.after_execute(&mut exec_result);
        assert_eq!(exec_result, expected);
    }

    #[test]
    fn ignore_tail_exceeds_length() {
        let input = "SORT_RESULT 0 10000";
        let interceptor = SortResultInterceptorFactory.try_new(input).unwrap();

        let mut exec_result = String::from(
            "3\
            \n2\
            \n1",
        );
        let expected = exec_result.clone();
        interceptor.after_execute(&mut exec_result);
        assert_eq!(exec_result, expected);
    }
}
