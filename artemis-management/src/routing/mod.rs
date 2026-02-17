//! Service routing engine and strategies

pub mod context;
pub mod engine;
pub mod strategy;

pub use context::RouteContext;
pub use engine::RouteEngine;
pub use strategy::{CloseByVisitStrategy, RouteStrategy, WeightedRoundRobinStrategy};
