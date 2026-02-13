use super::service::ServiceGroup;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteRule {
    pub route_id: String,
    pub strategy: RouteStrategy,
    pub groups: Vec<ServiceGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RouteStrategy {
    WeightedRoundRobin,
    CloseByVisit,
}
