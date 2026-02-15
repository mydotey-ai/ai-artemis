//! VersionedCacheManager 测试
//!
//! 测试覆盖:
//! - 版本化缓存机制
//! - 缓存更新和失效
//! - 并发缓存访问
//! - 增量差异计算
//! - 边界条件和异常场景

use artemis_core::model::{ChangeType, Instance, InstanceStatus, Service};
use artemis_server::cache::VersionedCacheManager;
use std::sync::Arc;

/// 创建测试服务
fn create_test_service(service_id: &str, instance_count: usize) -> Service {
    let instances = (0..instance_count)
        .map(|i| create_test_instance(service_id, &format!("inst-{}", i)))
        .collect();

    Service {
        service_id: service_id.to_string(),
        metadata: None,
        instances,
        logic_instances: None,
        route_rules: None,
    }
}

/// 创建测试实例
fn create_test_instance(service_id: &str, instance_id: &str) -> Instance {
    Instance {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
        group_id: Some("default".to_string()),
        service_id: service_id.to_string(),
        instance_id: instance_id.to_string(),
        machine_name: None,
        ip: "192.168.1.100".to_string(),
        port: 8080,
        protocol: None,
        url: "http://192.168.1.100:8080".to_string(),
        health_check_url: None,
        status: InstanceStatus::Up,
        metadata: None,
    }
}

// ===== 版本化缓存机制测试 =====

#[test]
fn test_version_starts_at_zero() {
    let manager = VersionedCacheManager::new();
    assert_eq!(manager.get_version(), 0, "初始版本应该为 0");
}

#[test]
fn test_version_increments_on_update() {
    let manager = VersionedCacheManager::new();
    let service = create_test_service("my-service", 1);

    let v0 = manager.get_version();
    manager.update_service(service);

    assert_eq!(manager.get_version(), v0 + 1, "更新服务后版本应该递增");
}

#[test]
fn test_version_increments_on_remove() {
    let manager = VersionedCacheManager::new();
    let service = create_test_service("my-service", 1);

    manager.update_service(service);
    let v1 = manager.get_version();

    manager.remove_service("my-service");
    assert_eq!(manager.get_version(), v1 + 1, "删除服务后版本应该递增");
}

#[test]
fn test_version_increments_on_clear() {
    let manager = VersionedCacheManager::new();
    manager.update_service(create_test_service("service-1", 1));
    manager.update_service(create_test_service("service-2", 1));

    let v2 = manager.get_version();
    manager.clear();

    assert_eq!(manager.get_version(), v2 + 1, "清空缓存后版本应该递增");
}

#[test]
fn test_multiple_updates_increment_version_sequentially() {
    let manager = VersionedCacheManager::new();

    let v0 = manager.get_version();
    manager.update_service(create_test_service("service-1", 1));
    assert_eq!(manager.get_version(), v0 + 1);

    manager.update_service(create_test_service("service-2", 1));
    assert_eq!(manager.get_version(), v0 + 2);

    manager.update_service(create_test_service("service-3", 1));
    assert_eq!(manager.get_version(), v0 + 3);

    manager.remove_service("service-2");
    assert_eq!(manager.get_version(), v0 + 4);
}

// ===== 缓存更新和失效测试 =====

#[test]
fn test_update_and_get_service() {
    let manager = VersionedCacheManager::new();
    let service = create_test_service("my-service", 2);

    manager.update_service(service.clone());

    let cached = manager.get_service("my-service");
    assert!(cached.is_some(), "应该能获取到缓存的服务");
    assert_eq!(cached.unwrap().service_id, "my-service");
}

#[test]
fn test_get_service_case_insensitive() {
    let manager = VersionedCacheManager::new();
    let service = create_test_service("My-Service", 1);

    manager.update_service(service);

    assert!(manager.get_service("my-service").is_some(), "小写查询应该成功");
    assert!(manager.get_service("My-Service").is_some(), "大写查询应该成功");
    assert!(manager.get_service("MY-SERVICE").is_some(), "全大写查询应该成功");
}

#[test]
fn test_update_replaces_existing_service() {
    let manager = VersionedCacheManager::new();

    let service_v1 = create_test_service("my-service", 1);
    manager.update_service(service_v1);

    let service_v2 = create_test_service("my-service", 3);
    manager.update_service(service_v2);

    let cached = manager.get_service("my-service").unwrap();
    assert_eq!(cached.instances.len(), 3, "应该使用最新的服务数据");
}

#[test]
fn test_remove_service_deletes_from_cache() {
    let manager = VersionedCacheManager::new();
    let service = create_test_service("my-service", 1);

    manager.update_service(service);
    assert!(manager.get_service("my-service").is_some(), "服务应该存在");

    manager.remove_service("my-service");
    assert!(manager.get_service("my-service").is_none(), "服务应该被删除");
}

#[test]
fn test_clear_removes_all_services() {
    let manager = VersionedCacheManager::new();

    manager.update_service(create_test_service("service-1", 1));
    manager.update_service(create_test_service("service-2", 1));
    manager.update_service(create_test_service("service-3", 1));

    assert_eq!(manager.get_all_services().len(), 3, "应该有 3 个服务");

    manager.clear();
    assert_eq!(manager.get_all_services().len(), 0, "清空后应该没有服务");
}

#[test]
fn test_get_nonexistent_service_returns_none() {
    let manager = VersionedCacheManager::new();
    assert!(manager.get_service("nonexistent").is_none(), "不存在的服务应该返回 None");
}

#[test]
fn test_get_all_services_returns_all_cached_services() {
    let manager = VersionedCacheManager::new();

    manager.update_service(create_test_service("service-1", 1));
    manager.update_service(create_test_service("service-2", 2));
    manager.update_service(create_test_service("service-3", 3));

    let all = manager.get_all_services();
    assert_eq!(all.len(), 3, "应该返回所有 3 个服务");

    let service_ids: Vec<String> = all.iter().map(|s| s.service_id.clone()).collect();
    assert!(service_ids.contains(&"service-1".to_string()));
    assert!(service_ids.contains(&"service-2".to_string()));
    assert!(service_ids.contains(&"service-3".to_string()));
}

#[test]
fn test_get_all_services_on_empty_cache_returns_empty_vec() {
    let manager = VersionedCacheManager::new();
    let all = manager.get_all_services();
    assert_eq!(all.len(), 0, "空缓存应该返回空列表");
}

// ===== 并发缓存访问测试 =====

#[test]
fn test_concurrent_updates() {
    use std::thread;

    let manager = Arc::new(VersionedCacheManager::new());
    let mut handles = vec![];

    // 10 个线程并发更新不同的服务
    for i in 0..10 {
        let mgr = manager.clone();
        let handle = thread::spawn(move || {
            let service = create_test_service(&format!("service-{}", i), 1);
            mgr.update_service(service);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(manager.get_all_services().len(), 10, "应该有 10 个服务");
    assert_eq!(manager.get_version(), 10, "版本应该递增 10 次");
}

#[test]
fn test_concurrent_reads() {
    use std::thread;

    let manager = Arc::new(VersionedCacheManager::new());

    // 先添加一些服务
    for i in 0..5 {
        manager.update_service(create_test_service(&format!("service-{}", i), 1));
    }

    let mut handles = vec![];

    // 10 个线程并发读取
    for _ in 0..10 {
        let mgr = manager.clone();
        let handle = thread::spawn(move || {
            for i in 0..5 {
                let service = mgr.get_service(&format!("service-{}", i));
                assert!(service.is_some(), "应该能读取到服务");
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

#[test]
fn test_concurrent_update_same_service() {
    use std::thread;

    let manager = Arc::new(VersionedCacheManager::new());
    let mut handles = vec![];

    // 10 个线程并发更新同一个服务
    for i in 0..10 {
        let mgr = manager.clone();
        let handle = thread::spawn(move || {
            let service = create_test_service("my-service", i);
            mgr.update_service(service);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // 最终应该只有 1 个服务,版本递增 10 次
    assert_eq!(manager.get_all_services().len(), 1, "应该只有 1 个服务");
    assert_eq!(manager.get_version(), 10, "版本应该递增 10 次");
}

#[test]
fn test_concurrent_update_and_read() {
    use std::thread;

    let manager = Arc::new(VersionedCacheManager::new());
    manager.update_service(create_test_service("my-service", 1));

    let mut handles = vec![];

    // 5 个写线程
    for _ in 0..5 {
        let mgr = manager.clone();
        let handle = thread::spawn(move || {
            for i in 0..10 {
                let service = create_test_service(&format!("service-{}", i), 1);
                mgr.update_service(service);
            }
        });
        handles.push(handle);
    }

    // 5 个读线程
    for _ in 0..5 {
        let mgr = manager.clone();
        let handle = thread::spawn(move || {
            for _ in 0..20 {
                let _ = mgr.get_all_services();
                let _ = mgr.get_version();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // 验证最终状态一致
    assert!(manager.get_all_services().len() > 0, "应该有服务存在");
}

// ===== 增量差异计算测试 =====

#[test]
fn test_compute_delta_new_service() {
    let old_services = vec![];
    let new_services = vec![create_test_service("my-service", 2)];

    let delta = VersionedCacheManager::compute_delta(&old_services, &new_services);

    assert_eq!(delta.len(), 1, "应该有 1 个服务的变更");
    let changes = delta.get("my-service").unwrap();
    assert_eq!(changes.len(), 2, "应该有 2 个新实例");
    assert!(changes.iter().all(|c| c.change_type == ChangeType::New));
}

#[test]
fn test_compute_delta_deleted_service() {
    let old_services = vec![create_test_service("my-service", 2)];
    let new_services = vec![];

    let delta = VersionedCacheManager::compute_delta(&old_services, &new_services);

    assert_eq!(delta.len(), 1, "应该有 1 个服务的变更");
    let changes = delta.get("my-service").unwrap();
    assert_eq!(changes.len(), 2, "应该有 2 个删除的实例");
    assert!(changes.iter().all(|c| c.change_type == ChangeType::Delete));
}

#[test]
fn test_compute_delta_instance_added() {
    let old_services = vec![create_test_service("my-service", 2)];
    let new_services = vec![create_test_service("my-service", 3)];

    let delta = VersionedCacheManager::compute_delta(&old_services, &new_services);

    assert_eq!(delta.len(), 1, "应该有 1 个服务的变更");
    let changes = delta.get("my-service").unwrap();
    assert_eq!(changes.len(), 1, "应该有 1 个新增实例");
    assert_eq!(changes[0].change_type, ChangeType::New);
}

#[test]
fn test_compute_delta_instance_removed() {
    let old_services = vec![create_test_service("my-service", 3)];
    let new_services = vec![create_test_service("my-service", 2)];

    let delta = VersionedCacheManager::compute_delta(&old_services, &new_services);

    assert_eq!(delta.len(), 1, "应该有 1 个服务的变更");
    let changes = delta.get("my-service").unwrap();
    assert_eq!(changes.len(), 1, "应该有 1 个删除的实例");
    assert_eq!(changes[0].change_type, ChangeType::Delete);
}

#[test]
fn test_compute_delta_instance_changed() {
    let old_service = create_test_service("my-service", 1);
    let mut new_service = create_test_service("my-service", 1);

    // 修改实例状态
    new_service.instances[0].status = InstanceStatus::Down;

    let delta = VersionedCacheManager::compute_delta(&[old_service], &[new_service]);

    assert_eq!(delta.len(), 1, "应该有 1 个服务的变更");
    let changes = delta.get("my-service").unwrap();
    assert_eq!(changes.len(), 1, "应该有 1 个变更的实例");
    assert_eq!(changes[0].change_type, ChangeType::Change);
}

#[test]
fn test_compute_delta_no_changes() {
    let old_services = vec![create_test_service("my-service", 2)];
    let new_services = vec![create_test_service("my-service", 2)];

    let delta = VersionedCacheManager::compute_delta(&old_services, &new_services);

    assert_eq!(delta.len(), 0, "没有变更时应该返回空 delta");
}

#[test]
fn test_compute_delta_multiple_services() {
    let old_services = vec![
        create_test_service("service-1", 2),
        create_test_service("service-2", 1),
    ];

    let new_services = vec![
        create_test_service("service-1", 3), // 新增 1 个实例
        create_test_service("service-3", 1), // 新服务
    ];

    let delta = VersionedCacheManager::compute_delta(&old_services, &new_services);

    assert_eq!(delta.len(), 3, "应该有 3 个服务的变更");

    // service-1: 新增 1 个实例
    let changes_1 = delta.get("service-1").unwrap();
    assert_eq!(changes_1.len(), 1);
    assert_eq!(changes_1[0].change_type, ChangeType::New);

    // service-2: 删除所有实例
    let changes_2 = delta.get("service-2").unwrap();
    assert_eq!(changes_2.len(), 1);
    assert_eq!(changes_2[0].change_type, ChangeType::Delete);

    // service-3: 新服务
    let changes_3 = delta.get("service-3").unwrap();
    assert_eq!(changes_3.len(), 1);
    assert_eq!(changes_3[0].change_type, ChangeType::New);
}

// ===== 边界条件和异常场景测试 =====

#[test]
fn test_default_constructor() {
    let manager = VersionedCacheManager::default();
    assert_eq!(manager.get_version(), 0, "默认构造器应该初始化版本为 0");
    assert_eq!(manager.get_all_services().len(), 0, "默认构造器应该初始化空缓存");
}

#[test]
fn test_clone_shares_state() {
    let manager1 = VersionedCacheManager::new();
    manager1.update_service(create_test_service("my-service", 1));

    let manager2 = manager1.clone();

    assert_eq!(manager2.get_version(), manager1.get_version(), "克隆应该共享版本");
    assert!(manager2.get_service("my-service").is_some(), "克隆应该共享缓存");

    // 在 manager2 中更新
    manager2.update_service(create_test_service("another-service", 1));

    // manager1 也应该看到变更
    assert!(manager1.get_service("another-service").is_some(), "克隆共享状态");
    assert_eq!(manager1.get_version(), manager2.get_version(), "版本应该同步");
}

#[test]
fn test_remove_nonexistent_service_is_safe() {
    let manager = VersionedCacheManager::new();
    let v0 = manager.get_version();

    manager.remove_service("nonexistent");

    // 删除不存在的服务仍然会递增版本
    assert_eq!(manager.get_version(), v0 + 1, "版本应该递增");
}

#[test]
fn test_empty_service_list() {
    let manager = VersionedCacheManager::new();
    let empty_service = Service {
        service_id: "empty-service".to_string(),
        metadata: None,
        instances: vec![],
        logic_instances: None,
        route_rules: None,
    };

    manager.update_service(empty_service);

    let cached = manager.get_service("empty-service").unwrap();
    assert_eq!(cached.instances.len(), 0, "应该支持空实例列表");
}

#[test]
fn test_compute_delta_with_empty_lists() {
    let delta = VersionedCacheManager::compute_delta(&[], &[]);
    assert_eq!(delta.len(), 0, "空列表应该返回空 delta");
}

#[test]
fn test_version_overflow_safety() {
    let manager = VersionedCacheManager::new();

    // 验证版本可以递增很多次
    for i in 0..1000 {
        manager.update_service(create_test_service(&format!("service-{}", i % 100), 1));
    }

    assert_eq!(manager.get_version(), 1000, "版本应该正确递增到 1000");
}
