// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

//! Query interceptor implementations.

use std::{collections::HashMap, sync::Arc};

use crate::{
    case::QueryContext,
    interceptor::{
        arg::ArgInterceptorFactory, env::EnvInterceptorFactory, replace::ReplaceInterceptorFactory,
        sort_result::SortResultInterceptorFactory, template::TemplateInterceptorFactory,
    },
};

pub mod arg;
pub mod env;
pub mod replace;
pub mod sort_result;
pub mod template;

pub type InterceptorRef = Box<dyn Interceptor>;

pub trait Interceptor {
    #[allow(unused_variables)]
    fn before_execute(&self, execute_query: &mut Vec<String>, context: &mut QueryContext) {}

    #[allow(unused_variables)]
    fn after_execute(&self, result: &mut String) {}
}

pub type InterceptorFactoryRef = Arc<dyn InterceptorFactory>;

pub trait InterceptorFactory {
    fn try_new(&self, interceptor: &str) -> Option<InterceptorRef>;
}

/// Interceptors builtin sqlness
pub fn builtin_interceptors() -> HashMap<String, InterceptorFactoryRef> {
    [
        (
            arg::PREFIX.to_string(),
            Arc::new(ArgInterceptorFactory {}) as InterceptorFactoryRef,
        ),
        // Arc::new(ArgInterceptorFactory {}),
        // Arc::new(ReplaceInterceptorFactory {}),
        // Arc::new(EnvInterceptorFactory {}),
        // Arc::new(SortResultInterceptorFactory {}),
        // Arc::new(TemplateInterceptorFactory {}),
    ]
    .into_iter()
    .map(|(prefix, factory)| (prefix.to_string(), factory))
    .collect()
}
