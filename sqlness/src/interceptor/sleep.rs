// Copyright 2024 CeresDB Project Authors. Licensed under Apache-2.0.

use std::pin::Pin;
use std::task::Context;
use std::time::{Duration, Instant};

use crate::error::Result;
use crate::interceptor::{Interceptor, InterceptorFactory, InterceptorRef};
use crate::SqlnessError;

pub const PREFIX: &str = "SLEEP";

/// Sleep for given duration before executing the query.
///
/// # Example
/// ``` sql
/// -- SQLNESS SLEEP <Duration>
/// SELECT 1;
/// ```
///
/// valid duration format:
/// - `1s` for 1 second
/// - `1ms` for 1 millisecond
/// - `1s500ms` for 1.5 seconds
/// etc. See detailed format in [duration_str](https://docs.rs/duration-str/0.11.2/duration_str/) crate
///
/// Note that this implementation is not accurate and may be affected by the system load.
/// It is guaranteed that the sleep time is at least the given milliseconds, but the lag may be
/// longer.
#[derive(Debug)]
pub struct SleepInterceptor {
    duration: Duration,
}

struct Sleep {
    now: Instant,
    duration: Duration,
}
impl core::future::Future for Sleep {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> std::task::Poll<Self::Output> {
        let elapsed = self.now.elapsed();
        if elapsed < self.duration {
            let waker = cx.waker().clone();
            // detach the thread and let it wake the waker later
            let remaining = self.duration.saturating_sub(elapsed);
            std::thread::spawn(move || {
                std::thread::sleep(remaining);
                waker.wake();
            });
            std::task::Poll::Pending
        } else {
            std::task::Poll::Ready(())
        }
    }
}

#[async_trait::async_trait]
impl Interceptor for SleepInterceptor {
    async fn before_execute_async(
        &self,
        _execute_query: &mut Vec<String>,
        _context: &mut crate::case::QueryContext,
    ) {
        // impl a cross-runtime sleep
        Sleep {
            now: Instant::now(),
            duration: self.duration,
        }
        .await;
    }
}

pub struct SleepInterceptorFactory;

impl InterceptorFactory for SleepInterceptorFactory {
    fn try_new(&self, ctx: &str) -> Result<InterceptorRef> {
        let duration = duration_str::parse(ctx).map_err(|e| SqlnessError::InvalidContext {
            prefix: PREFIX.to_string(),
            msg: format!("Failed to parse duration: {}", e),
        })?;
        Ok(Box::new(SleepInterceptor { duration }))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn wait_1500ms() {
        let input = "1s500ms";
        let interceptor = SleepInterceptorFactory {}.try_new(input).unwrap();
        let now = Instant::now();
        interceptor
            .before_execute_async(&mut vec![], &mut crate::QueryContext::default())
            .await;
        let elasped = now.elapsed().as_millis() as u64;
        assert!(elasped >= 1500);
    }
}
