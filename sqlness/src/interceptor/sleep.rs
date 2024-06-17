// Copyright 2024 CeresDB Project Authors. Licensed under Apache-2.0.

use std::pin::Pin;
use std::task::Context;
use std::time::{Duration, Instant};

use crate::error::Result;
use crate::interceptor::{Interceptor, InterceptorFactory, InterceptorRef};
use crate::SqlnessError;

pub const PREFIX: &str = "SLEEP";

/// Sleep for given milliseconds before executing the query.
///
/// # Example
/// ``` sql
/// -- SQLNESS SLEEP 1500
/// SELECT 1;
/// ```
///
/// Note that this implementation is not accurate and may be affected by the system load.
/// It is guaranteed that the sleep time is at least the given milliseconds, but the lag may be
/// longer.
#[derive(Debug)]
pub struct SleepInterceptor {
    milliseconds: u64,
}

#[async_trait::async_trait]
impl Interceptor for SleepInterceptor {
    async fn before_execute_async(
        &self,
        _execute_query: &mut Vec<String>,
        _context: &mut crate::case::QueryContext,
    ) {
        // impl a cross-runtime sleep
        struct Sleep {
            now: Instant,
            milliseconds: u64,
        }
        impl core::future::Future for Sleep {
            type Output = ();
            fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> std::task::Poll<Self::Output> {
                let elapsed = self.now.elapsed().as_millis() as u64;
                let remaining = self.milliseconds.saturating_sub(elapsed);
                if elapsed < self.milliseconds {
                    let waker = cx.waker().clone();
                    // detach the thread and let it wake the waker later
                    std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(remaining));
                        waker.wake();
                    });
                    std::task::Poll::Pending
                } else {
                    std::task::Poll::Ready(())
                }
            }
        }
        Sleep {
            now: Instant::now(),
            milliseconds: self.milliseconds,
        }
        .await;
    }
}

pub struct SleepInterceptorFactory;

impl InterceptorFactory for SleepInterceptorFactory {
    fn try_new(&self, ctx: &str) -> Result<InterceptorRef> {
        let milliseconds = ctx
            .parse::<u64>()
            .map_err(|e| SqlnessError::InvalidContext {
                prefix: PREFIX.to_string(),
                msg: format!("Failed to parse milliseconds: {}", e),
            })?;
        Ok(Box::new(SleepInterceptor { milliseconds }))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn wait_1500ms() {
        let input = "1500";
        let interceptor = SleepInterceptorFactory {}.try_new(input).unwrap();
        let now = Instant::now();
        interceptor
            .before_execute_async(&mut vec![], &mut crate::QueryContext::default())
            .await;
        let elasped = now.elapsed().as_millis() as u64;
        assert!(elasped >= 1500);
    }
}
