use super::instance::Instance;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceChange {
    pub instance: Instance,
    pub change_type: ChangeType,
    pub change_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChangeType {
    New,
    Delete,
    Change,
    Reload,
}
