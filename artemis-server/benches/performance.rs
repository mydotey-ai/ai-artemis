//! Performance benchmarks for Artemis Server
//!
//! Run with: cargo bench

use artemis_core::model::{
    HeartbeatRequest, Instance, InstanceKey, InstanceStatus, RegisterRequest,
};
use artemis_core::traits::RegistryService;
use artemis_server::{
    cache::VersionedCacheManager, change::InstanceChangeManager, RegistryServiceImpl,
    lease::LeaseManager, registry::RegistryRepository,
};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::sync::Arc;
use std::time::Duration;

fn create_test_instance(id: usize) -> Instance {
    Instance {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
        service_id: "benchmark-service".to_string(),
        group_id: None,
        instance_id: format!("inst-{}", id),
        machine_name: None,
        ip: format!("192.168.1.{}", id % 255),
        port: 8080,
        protocol: None,
        url: format!("http://192.168.1.{}:8080", id % 255),
        health_check_url: None,
        status: InstanceStatus::Up,
        metadata: None,
    }
}

fn bench_register(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("register");

    for size in [1, 10, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                rt.block_on(async {
                    let repo = RegistryRepository::new();
                    let lease_mgr = Arc::new(LeaseManager::new(Duration::from_secs(30)));
                    let cache = Arc::new(VersionedCacheManager::new());
                    let change_mgr = Arc::new(InstanceChangeManager::new());
                    let service = RegistryServiceImpl::new(
                        repo,
                        lease_mgr,
                        cache,
                        change_mgr,
                        None, // No replication in benchmark
                    );

                    let instances: Vec<Instance> = (0..size).map(create_test_instance).collect();
                    let request = RegisterRequest { instances };

                    black_box(service.register(request).await);
                });
            });
        });
    }

    group.finish();
}

fn bench_heartbeat(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("heartbeat");

    for size in [1, 10, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                rt.block_on(async {
                    let repo = RegistryRepository::new();
                    let lease_mgr = Arc::new(LeaseManager::new(Duration::from_secs(30)));
                    let cache = Arc::new(VersionedCacheManager::new());
                    let change_mgr = Arc::new(InstanceChangeManager::new());
                    let service = RegistryServiceImpl::new(
                        repo.clone(),
                        lease_mgr.clone(),
                        cache,
                        change_mgr,
                        None, // No replication in benchmark
                    );

                    // 先注册实例
                    let instances: Vec<Instance> = (0..size).map(create_test_instance).collect();
                    service.register(RegisterRequest { instances: instances.clone() }).await;

                    // 心跳测试
                    let keys: Vec<InstanceKey> = instances.iter().map(|i| i.key()).collect();
                    let request = HeartbeatRequest { instance_keys: keys };

                    black_box(service.heartbeat(request).await);
                });
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_register, bench_heartbeat);
criterion_main!(benches);
