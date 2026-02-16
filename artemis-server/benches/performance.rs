//! Performance benchmarks for Artemis Server
//!
//! Run with: cargo bench

use artemis_core::model::{
    DiscoveryConfig, GetServiceRequest, HeartbeatRequest, Instance, InstanceKey, InstanceStatus,
    RegisterRequest, Service,
};
use artemis_core::traits::{DiscoveryService, RegistryService};
use artemis_server::{
    cache::VersionedCacheManager, change::InstanceChangeManager, RegistryServiceImpl,
    discovery::DiscoveryServiceImpl, lease::LeaseManager, registry::RegistryRepository,
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

fn bench_discovery(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("discovery");

    for size in [1, 10, 100].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                rt.block_on(async {
                    let repo = RegistryRepository::new();
                    let lease_mgr = Arc::new(LeaseManager::new(Duration::from_secs(30)));
                    let cache = Arc::new(VersionedCacheManager::new());
                    let change_mgr = Arc::new(InstanceChangeManager::new());
                    let reg_service = RegistryServiceImpl::new(
                        repo.clone(),
                        lease_mgr.clone(),
                        cache.clone(),
                        change_mgr,
                        None,
                    );

                    // 先注册实例
                    let instances: Vec<Instance> = (0..size).map(create_test_instance).collect();
                    reg_service.register(RegisterRequest { instances }).await;

                    // 服务发现测试
                    let discovery = DiscoveryServiceImpl::new(
                        repo,
                        cache,
                    );

                    let config = DiscoveryConfig {
                        service_id: "benchmark-service".to_string(),
                        region_id: "test-region".to_string(),
                        zone_id: "test-zone".to_string(),
                        discovery_data: None,
                    };

                    let request = GetServiceRequest {
                        discovery_config: config,
                    };

                    black_box(discovery.get_service(request).await);
                });
            });
        });
    }

    group.finish();
}

fn bench_concurrent_register(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_register");

    for threads in [1, 5, 10].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(threads), threads, |b, &threads| {
            b.iter(|| {
                rt.block_on(async {
                    let repo = RegistryRepository::new();
                    let lease_mgr = Arc::new(LeaseManager::new(Duration::from_secs(30)));
                    let cache = Arc::new(VersionedCacheManager::new());
                    let change_mgr = Arc::new(InstanceChangeManager::new());
                    let service = Arc::new(RegistryServiceImpl::new(
                        repo,
                        lease_mgr,
                        cache,
                        change_mgr,
                        None,
                    ));

                    let mut handles = vec![];
                    for i in 0..threads {
                        let service = service.clone();
                        let handle = tokio::spawn(async move {
                            let instances = vec![create_test_instance(i * 100)];
                            let request = RegisterRequest { instances };
                            service.register(request).await
                        });
                        handles.push(handle);
                    }

                    for handle in handles {
                        let _ = black_box(handle.await);
                    }
                });
            });
        });
    }

    group.finish();
}

fn bench_cache_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("cache_operations");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                rt.block_on(async {
                    let cache = VersionedCacheManager::new();
                    let instances: Vec<Instance> = (0..size).map(create_test_instance).collect();

                    // 测试更新和查询操作
                    let service = Service {
                        service_id: "benchmark-service".to_string(),
                        metadata: None,
                        instances: instances.clone(),
                        logic_instances: None,
                        route_rules: None,
                    };
                    cache.update_service(service.clone());
                    let cached = cache.get_service("benchmark-service");
                    black_box(cached);

                    // 测试版本管理
                    let version = cache.get_version();
                    let all_services = cache.get_all_services();
                    black_box((version, all_services));
                });
            });
        });
    }

    group.finish();
}

fn bench_lease_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();

    let mut group = c.benchmark_group("lease_operations");

    for size in [10, 100, 1000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                rt.block_on(async {
                    let lease_mgr = LeaseManager::new(Duration::from_secs(30));

                    // 创建租约
                    for i in 0..size {
                        let instance = create_test_instance(i);
                        lease_mgr.create_lease(instance.key());
                    }

                    // 续约测试
                    for i in 0..size {
                        let instance = create_test_instance(i);
                        black_box(lease_mgr.renew(&instance.key()));
                    }

                    // 获取所有租约
                    black_box(lease_mgr.count());
                });
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_register,
    bench_heartbeat,
    bench_discovery,
    bench_concurrent_register,
    bench_cache_operations,
    bench_lease_operations
);
criterion_main!(benches);
