//! Instance management operations
use artemis_core::model::InstanceKey;

pub struct InstanceManager;

impl Default for InstanceManager {
    fn default() -> Self {
        Self::new()
    }
}

impl InstanceManager {
    pub fn new() -> Self {
        Self
    }

    pub fn pull_in(&self, _key: &InstanceKey) -> anyhow::Result<()> {
        Ok(())
    }

    pub fn pull_out(&self, _key: &InstanceKey) -> anyhow::Result<()> {
        Ok(())
    }
}
