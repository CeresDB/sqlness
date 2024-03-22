// Copyright 2022 CeresDB Project Authors. Licensed under Apache-2.0.

//! Query interceptor implementations.

use std::sync::Arc;

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
pub fn builtin_interceptors() -> Vec<InterceptorFactoryRef> {
    vec![
        Arc::new(ArgInterceptorFactory {}),
        Arc::new(ReplaceInterceptorFactory {}),
        Arc::new(EnvInterceptorFactory {}),
        Arc::new(SortResultInterceptorFactory {}),
        Arc::new(TemplateInterceptorFactory {}),
    ]
}
