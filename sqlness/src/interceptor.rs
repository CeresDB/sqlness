// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

//! Query interceptor implementations.

use std::{collections::HashMap, sync::Arc};

use crate::{
    case::QueryContext,
    error::Result,
    error::SqlnessError,
    interceptor::{
        arg::ArgInterceptorFactory, env::EnvInterceptorFactory, replace::ReplaceInterceptorFactory,
        sort_result::SortResultInterceptorFactory, template::TemplateInterceptorFactory,
    },
};

pub mod arg;
pub mod env;
pub mod replace;
pub mod sleep;
pub mod sort_result;
pub mod template;

pub type InterceptorRef = Box<dyn Interceptor + Send + Sync>;

#[async_trait::async_trait]
pub trait Interceptor {
    #[allow(unused_variables)]
    async fn before_execute(&self, execute_query: &mut Vec<String>, context: &mut QueryContext) {}

    #[allow(unused_variables)]
    async fn after_execute(&self, result: &mut String) {}
}

pub type InterceptorFactoryRef = Arc<dyn InterceptorFactory>;

pub trait InterceptorFactory {
    fn try_new(&self, ctx: &str) -> Result<InterceptorRef>;
}

#[derive(Clone)]
pub struct Registry {
    factories: HashMap<String, InterceptorFactoryRef>,
}

impl Default for Registry {
    fn default() -> Self {
        Self {
            factories: builtin_interceptors(),
        }
    }
}

impl Registry {
    pub fn register(&mut self, prefix: &str, factory: InterceptorFactoryRef) {
        self.factories.insert(prefix.to_string(), factory);
    }

    pub fn create(&self, ctx: &str) -> Result<InterceptorRef> {
        let mut args = ctx.trim().splitn(2, ' ');
        let prefix = args.next().ok_or_else(|| SqlnessError::MissingPrefix {
            line: ctx.to_string(),
        })?;
        let context = args.next().unwrap_or_default();
        if let Some(factory) = self.factories.get(prefix.trim()) {
            factory.try_new(context.trim())
        } else {
            Err(SqlnessError::UnknownInterceptor {
                prefix: prefix.to_string(),
            })
        }
    }
}

/// Interceptors builtin sqlness
fn builtin_interceptors() -> HashMap<String, InterceptorFactoryRef> {
    [
        (
            arg::PREFIX.to_string(),
            Arc::new(ArgInterceptorFactory {}) as _,
        ),
        (
            replace::PREFIX.to_string(),
            Arc::new(ReplaceInterceptorFactory {}) as _,
        ),
        (
            env::PREFIX.to_string(),
            Arc::new(EnvInterceptorFactory {}) as _,
        ),
        (
            sort_result::PREFIX.to_string(),
            Arc::new(SortResultInterceptorFactory {}) as _,
        ),
        (
            template::PREFIX.to_string(),
            Arc::new(TemplateInterceptorFactory {}) as _,
        ),
        (
            sleep::PREFIX.to_string(),
            Arc::new(sleep::SleepInterceptorFactory {}) as _,
        ),
    ]
    .into_iter()
    .map(|(prefix, factory)| (prefix.to_string(), factory))
    .collect()
}
