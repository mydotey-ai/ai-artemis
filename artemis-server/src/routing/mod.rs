//! Service routing engine and strategies

pub mod context;
pub mod strategy;
pub mod engine;

pub use context::RouteContext;
pub use strategy::{RouteStrategy, WeightedRoundRobinStrategy, CloseByVisitStrategy};
pub use engine::RouteEngine;
