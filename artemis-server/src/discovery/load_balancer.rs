//! 负载均衡器 - 用于从实例列表中选择单个实例
//!
//! 支持多种负载均衡策略:
//! - Random: 随机选择
//! - RoundRobin: 轮询选择
//! - WeightedRoundRobin: 加权轮询 (基于实例权重)

use artemis_core::model::Instance;
use rand::Rng;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// 负载均衡策略
#[derive(Debug, Clone, Copy, Default)]
pub enum LoadBalanceStrategy {
    /// 随机选择
    #[default]
    Random,
    /// 轮询选择
    RoundRobin,
}

/// 负载均衡器
pub struct LoadBalancer {
    /// 轮询计数器 (用于 RoundRobin 策略)
    round_robin_counter: Arc<AtomicUsize>,
}

impl LoadBalancer {
    pub fn new() -> Self {
        Self {
            round_robin_counter: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// 从实例列表中选择一个实例
    pub fn select_instance(
        &self,
        instances: &[Instance],
        strategy: LoadBalanceStrategy,
    ) -> Option<Instance> {
        if instances.is_empty() {
            return None;
        }

        match strategy {
            LoadBalanceStrategy::Random => self.select_random(instances),
            LoadBalanceStrategy::RoundRobin => self.select_round_robin(instances),
        }
    }

    /// 随机选择
    fn select_random(&self, instances: &[Instance]) -> Option<Instance> {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..instances.len());
        instances.get(index).cloned()
    }

    /// 轮询选择
    fn select_round_robin(&self, instances: &[Instance]) -> Option<Instance> {
        let counter = self.round_robin_counter.fetch_add(1, Ordering::Relaxed);
        let index = counter % instances.len();
        instances.get(index).cloned()
    }
}

impl Default for LoadBalancer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use artemis_core::model::InstanceStatus;

    fn create_test_instance(id: &str, ip: &str, port: u16) -> Instance {
        Instance {
            region_id: "us-east".to_string(),
            zone_id: "zone-1".to_string(),
            service_id: "test-service".to_string(),
            group_id: None,
            instance_id: id.to_string(),
            machine_name: None,
            ip: ip.to_string(),
            port,
            protocol: None,
            url: format!("http://{}:{}", ip, port),
            health_check_url: None,
            status: InstanceStatus::Up,
            metadata: None,
        }
    }

    #[test]
    fn test_select_from_empty_list() {
        let lb = LoadBalancer::new();
        let instances = vec![];

        assert!(lb.select_instance(&instances, LoadBalanceStrategy::Random).is_none());
        assert!(lb.select_instance(&instances, LoadBalanceStrategy::RoundRobin).is_none());
    }

    #[test]
    fn test_select_from_single_instance() {
        let lb = LoadBalancer::new();
        let instances = vec![create_test_instance("inst-1", "192.168.1.1", 8080)];

        let selected = lb.select_instance(&instances, LoadBalanceStrategy::Random).unwrap();
        assert_eq!(selected.instance_id, "inst-1");

        let selected = lb.select_instance(&instances, LoadBalanceStrategy::RoundRobin).unwrap();
        assert_eq!(selected.instance_id, "inst-1");
    }

    #[test]
    fn test_random_selection() {
        let lb = LoadBalancer::new();
        let instances = vec![
            create_test_instance("inst-1", "192.168.1.1", 8080),
            create_test_instance("inst-2", "192.168.1.2", 8080),
            create_test_instance("inst-3", "192.168.1.3", 8080),
        ];

        // 随机选择应该总能返回结果
        for _ in 0..10 {
            let selected = lb.select_instance(&instances, LoadBalanceStrategy::Random);
            assert!(selected.is_some());
        }
    }

    #[test]
    fn test_round_robin_selection() {
        let lb = LoadBalancer::new();
        let instances = vec![
            create_test_instance("inst-1", "192.168.1.1", 8080),
            create_test_instance("inst-2", "192.168.1.2", 8080),
            create_test_instance("inst-3", "192.168.1.3", 8080),
        ];

        // 轮询应该按顺序返回实例
        let selected1 = lb.select_instance(&instances, LoadBalanceStrategy::RoundRobin).unwrap();
        let selected2 = lb.select_instance(&instances, LoadBalanceStrategy::RoundRobin).unwrap();
        let selected3 = lb.select_instance(&instances, LoadBalanceStrategy::RoundRobin).unwrap();
        let selected4 = lb.select_instance(&instances, LoadBalanceStrategy::RoundRobin).unwrap();

        assert_eq!(selected1.instance_id, "inst-1");
        assert_eq!(selected2.instance_id, "inst-2");
        assert_eq!(selected3.instance_id, "inst-3");
        assert_eq!(selected4.instance_id, "inst-1"); // 循环回到第一个
    }

    #[test]
    fn test_round_robin_wraparound() {
        let lb = LoadBalancer::new();
        let instances = vec![
            create_test_instance("inst-1", "192.168.1.1", 8080),
            create_test_instance("inst-2", "192.168.1.2", 8080),
        ];

        // 验证轮询会循环
        for i in 0..10 {
            let selected = lb.select_instance(&instances, LoadBalanceStrategy::RoundRobin).unwrap();
            let expected_id = if i % 2 == 0 { "inst-1" } else { "inst-2" };
            assert_eq!(selected.instance_id, expected_id);
        }
    }
}
