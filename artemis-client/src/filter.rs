use artemis_core::model::{Instance, InstanceStatus};

/// Registry filter trait for filtering instance lists.
///
/// Implementations can filter instances based on any criteria
/// (status, metadata, zone, etc.)
pub trait RegistryFilter: Send + Sync {
    /// Filter instances, returning only those that pass the filter
    fn filter(&self, instances: Vec<Instance>) -> Vec<Instance>;

    /// Get the filter name (for logging)
    fn name(&self) -> &str;
}

/// Status filter - filters instances by their status.
///
/// Only instances whose status is in the `allowed_statuses` list
/// will pass through.
pub struct StatusFilter {
    allowed_statuses: Vec<InstanceStatus>,
}

impl StatusFilter {
    /// Create a new status filter with the allowed statuses
    pub fn new(allowed_statuses: Vec<InstanceStatus>) -> Self {
        Self { allowed_statuses }
    }
}

impl RegistryFilter for StatusFilter {
    fn filter(&self, instances: Vec<Instance>) -> Vec<Instance> {
        instances
            .into_iter()
            .filter(|inst| self.allowed_statuses.contains(&inst.status))
            .collect()
    }

    fn name(&self) -> &str {
        "StatusFilter"
    }
}

/// Filter chain - applies multiple filters in sequence.
///
/// Each filter is applied in order, with the output of one becoming
/// the input of the next.
pub struct FilterChain {
    filters: Vec<Box<dyn RegistryFilter>>,
}

impl FilterChain {
    /// Create an empty filter chain
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
        }
    }

    /// Add a filter to the chain
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, filter: Box<dyn RegistryFilter>) -> Self {
        self.filters.push(filter);
        self
    }

    /// Apply all filters in sequence
    pub fn apply(&self, instances: Vec<Instance>) -> Vec<Instance> {
        self.filters.iter().fold(instances, |acc, filter| {
            let before_count = acc.len();
            let filtered = filter.filter(acc);
            let after_count = filtered.len();

            if before_count != after_count {
                tracing::debug!(
                    "Filter '{}' reduced instances from {} to {}",
                    filter.name(),
                    before_count,
                    after_count
                );
            }

            filtered
        })
    }

    /// Check if the chain is empty (no filters added)
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }
}

impl Default for FilterChain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_instance(instance_id: &str, status: InstanceStatus) -> Instance {
        Instance {
            region_id: "region".into(),
            zone_id: "zone".into(),
            service_id: "service".into(),
            instance_id: instance_id.into(),
            ip: "127.0.0.1".into(),
            port: 8080,
            status,
            group_id: None,
            machine_name: None,
            protocol: None,
            url: "http://127.0.0.1:8080".into(),
            health_check_url: None,
            metadata: None,
        }
    }

    #[test]
    fn test_status_filter() {
        let filter = StatusFilter::new(vec![InstanceStatus::Up]);
        let instances = vec![
            make_test_instance("1", InstanceStatus::Up),
            make_test_instance("2", InstanceStatus::Down),
            make_test_instance("3", InstanceStatus::Up),
        ];

        let filtered = filter.filter(instances);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|i| i.status == InstanceStatus::Up));
    }

    #[test]
    fn test_filter_chain() {
        let chain = FilterChain::new()
            .add(Box::new(StatusFilter::new(vec![InstanceStatus::Up])));

        let instances = vec![
            make_test_instance("1", InstanceStatus::Up),
            make_test_instance("2", InstanceStatus::Down),
        ];

        let filtered = chain.apply(instances);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].instance_id, "1");
    }
}
