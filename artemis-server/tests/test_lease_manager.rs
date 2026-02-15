//! LeaseManager 边界测试
//!
//! 测试覆盖:
//! - 租约过期和自动清理
//! - TTL 更新和续约机制
//! - 并发租约操作
//! - 边界条件和异常场景

use artemis_core::model::InstanceKey;
use artemis_server::lease::LeaseManager;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

/// 创建测试用的 InstanceKey
fn create_test_key(service_id: &str, instance_id: &str) -> InstanceKey {
    InstanceKey {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
        service_id: service_id.to_string(),
        group_id: "default".to_string(),
        instance_id: instance_id.to_string(),
    }
}

// ===== 租约过期和自动清理测试 =====

#[tokio::test]
async fn test_lease_expires_after_ttl() {
    let manager = LeaseManager::new(Duration::from_millis(100));
    let key = create_test_key("my-service", "inst-1");

    manager.create_lease(key.clone());
    assert!(manager.is_valid(&key), "租约应该在创建后立即有效");

    // 等待 TTL 过期
    time::sleep(Duration::from_millis(150)).await;
    assert!(!manager.is_valid(&key), "租约应该在 TTL 后过期");
}

#[tokio::test]
async fn test_get_expired_keys_returns_expired_leases() {
    let manager = LeaseManager::new(Duration::from_millis(100));

    let key1 = create_test_key("service-1", "inst-1");
    let key2 = create_test_key("service-2", "inst-2");

    manager.create_lease(key1.clone());
    manager.create_lease(key2.clone());

    // 等待过期
    time::sleep(Duration::from_millis(150)).await;

    let expired = manager.get_expired_keys();
    assert_eq!(expired.len(), 2, "应该返回所有过期的租约");
}

#[tokio::test]
async fn test_eviction_task_automatically_removes_expired_leases() {
    let manager = LeaseManager::new(Duration::from_millis(100));
    let evicted_count = Arc::new(AtomicUsize::new(0));

    let key1 = create_test_key("service-1", "inst-1");
    let key2 = create_test_key("service-2", "inst-2");

    manager.create_lease(key1.clone());
    manager.create_lease(key2.clone());

    // 启动清理任务
    let counter = evicted_count.clone();
    manager.clone().start_eviction_task(
        Duration::from_millis(50),
        move |_key| {
            counter.fetch_add(1, Ordering::SeqCst);
        },
    );

    // 等待租约过期和清理任务运行
    time::sleep(Duration::from_millis(200)).await;

    assert_eq!(evicted_count.load(Ordering::SeqCst), 2, "应该清理掉2个过期租约");
    assert_eq!(manager.count(), 0, "清理后租约数量应为0");
}

#[tokio::test]
async fn test_eviction_task_does_not_remove_renewed_leases() {
    let manager = LeaseManager::new(Duration::from_millis(200));
    let evicted_count = Arc::new(AtomicUsize::new(0));

    let key = create_test_key("my-service", "inst-1");
    manager.create_lease(key.clone());

    // 启动清理任务
    let counter = evicted_count.clone();
    manager.clone().start_eviction_task(
        Duration::from_millis(50),
        move |_key| {
            counter.fetch_add(1, Ordering::SeqCst);
        },
    );

    // 定期续约
    for _ in 0..5 {
        time::sleep(Duration::from_millis(100)).await;
        manager.renew(&key);
    }

    // 验证租约仍然存在
    assert_eq!(evicted_count.load(Ordering::SeqCst), 0, "续约的租约不应该被清理");
    assert!(manager.is_valid(&key), "续约的租约应该保持有效");
}

// ===== TTL 更新和续约机制测试 =====

#[tokio::test]
async fn test_renew_extends_lease_lifetime() {
    let manager = LeaseManager::new(Duration::from_millis(100));
    let key = create_test_key("my-service", "inst-1");

    manager.create_lease(key.clone());

    // 等待80ms后续约
    time::sleep(Duration::from_millis(80)).await;
    assert!(manager.renew(&key), "续约应该成功");

    // 再等待80ms,如果没续约应该过期,但续约后不应过期
    time::sleep(Duration::from_millis(80)).await;
    assert!(manager.is_valid(&key), "续约后租约应该延长生命周期");

    // 再等待超过 TTL 的时间
    time::sleep(Duration::from_millis(50)).await;
    assert!(!manager.is_valid(&key), "超过新的 TTL 后应该过期");
}

#[tokio::test]
async fn test_renew_nonexistent_lease_returns_false() {
    let manager = LeaseManager::new(Duration::from_secs(30));
    let key = create_test_key("my-service", "nonexistent");

    assert!(!manager.renew(&key), "续约不存在的租约应该返回 false");
}

#[tokio::test]
async fn test_multiple_renewals_keep_lease_alive() {
    let manager = LeaseManager::new(Duration::from_millis(100));
    let key = create_test_key("my-service", "inst-1");

    manager.create_lease(key.clone());

    // 多次续约
    for _ in 0..10 {
        time::sleep(Duration::from_millis(50)).await;
        assert!(manager.renew(&key), "续约应该成功");
        assert!(manager.is_valid(&key), "租约应该保持有效");
    }
}

// ===== 并发租约操作测试 =====

#[tokio::test]
async fn test_concurrent_lease_creation() {
    let manager = Arc::new(LeaseManager::new(Duration::from_secs(30)));
    let mut handles = vec![];

    // 100个并发任务创建租约
    for i in 0..100 {
        let mgr = manager.clone();
        let handle = tokio::spawn(async move {
            let key = create_test_key("service", &format!("inst-{}", i));
            mgr.create_lease(key.clone());
            assert!(mgr.is_valid(&key));
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        handle.await.unwrap();
    }

    assert_eq!(manager.count(), 100, "应该创建100个租约");
}

#[tokio::test]
async fn test_concurrent_renewal() {
    let manager = Arc::new(LeaseManager::new(Duration::from_millis(200)));
    let key = create_test_key("my-service", "inst-1");

    manager.create_lease(key.clone());

    let mut handles = vec![];

    // 10个并发任务同时续约
    for _ in 0..10 {
        let mgr = manager.clone();
        let k = key.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..5 {
                assert!(mgr.renew(&k), "并发续约应该成功");
                time::sleep(Duration::from_millis(20)).await;
            }
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        handle.await.unwrap();
    }

    assert!(manager.is_valid(&key), "并发续约后租约应该仍然有效");
}

#[tokio::test]
async fn test_concurrent_removal() {
    let manager = Arc::new(LeaseManager::new(Duration::from_secs(30)));

    // 创建多个租约
    for i in 0..100 {
        let key = create_test_key("service", &format!("inst-{}", i));
        manager.create_lease(key);
    }

    let mut handles = vec![];

    // 并发删除租约
    for i in 0..100 {
        let mgr = manager.clone();
        let handle = tokio::spawn(async move {
            let key = create_test_key("service", &format!("inst-{}", i));
            mgr.remove_lease(&key);
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    for handle in handles {
        handle.await.unwrap();
    }

    assert_eq!(manager.count(), 0, "所有租约应该被删除");
}

// ===== 边界条件和异常场景测试 =====

#[tokio::test]
async fn test_remove_nonexistent_lease_returns_none() {
    let manager = LeaseManager::new(Duration::from_secs(30));
    let key = create_test_key("my-service", "nonexistent");

    assert!(manager.remove_lease(&key).is_none(), "删除不存在的租约应该返回 None");
}

#[tokio::test]
async fn test_is_valid_returns_false_for_nonexistent_lease() {
    let manager = LeaseManager::new(Duration::from_secs(30));
    let key = create_test_key("my-service", "nonexistent");

    assert!(!manager.is_valid(&key), "不存在的租约应该返回无效");
}

#[tokio::test]
async fn test_zero_ttl_lease_expires_immediately() {
    let manager = LeaseManager::new(Duration::from_millis(0));
    let key = create_test_key("my-service", "inst-1");

    manager.create_lease(key.clone());

    // 零 TTL 租约应该立即过期
    time::sleep(Duration::from_millis(1)).await;
    assert!(!manager.is_valid(&key), "零 TTL 租约应该立即过期");
}

#[tokio::test]
async fn test_removed_lease_not_in_expired_keys() {
    let manager = LeaseManager::new(Duration::from_millis(100));
    let key = create_test_key("my-service", "inst-1");

    manager.create_lease(key.clone());
    manager.remove_lease(&key);

    // 等待过期时间
    time::sleep(Duration::from_millis(150)).await;

    let expired = manager.get_expired_keys();
    assert_eq!(expired.len(), 0, "已删除的租约不应该出现在过期列表中");
}

#[tokio::test]
async fn test_get_all_leases_returns_all_active_leases() {
    let manager = LeaseManager::new(Duration::from_secs(30));

    let key1 = create_test_key("service-1", "inst-1");
    let key2 = create_test_key("service-2", "inst-2");

    manager.create_lease(key1.clone());
    manager.create_lease(key2.clone());

    let all_leases = manager.get_all_leases();
    assert_eq!(all_leases.len(), 2, "应该返回所有活跃租约");
}

#[tokio::test]
async fn test_get_all_leases_includes_expired_leases() {
    let manager = LeaseManager::new(Duration::from_millis(100));

    let key1 = create_test_key("service-1", "inst-1");
    let key2 = create_test_key("service-2", "inst-2");

    manager.create_lease(key1);
    manager.create_lease(key2);

    // 等待过期
    time::sleep(Duration::from_millis(150)).await;

    let all_leases = manager.get_all_leases();
    assert_eq!(all_leases.len(), 2, "get_all_leases 应该包含过期但未清理的租约");
}

#[tokio::test]
async fn test_lease_count_decreases_after_removal() {
    let manager = LeaseManager::new(Duration::from_secs(30));

    let key1 = create_test_key("service-1", "inst-1");
    let key2 = create_test_key("service-2", "inst-2");

    manager.create_lease(key1.clone());
    manager.create_lease(key2);

    assert_eq!(manager.count(), 2, "创建后应该有2个租约");

    manager.remove_lease(&key1);
    assert_eq!(manager.count(), 1, "删除后应该剩1个租约");
}

// ===== 租约状态检查测试 =====

#[tokio::test]
async fn test_lease_valid_immediately_after_creation() {
    let manager = LeaseManager::new(Duration::from_secs(30));
    let key = create_test_key("my-service", "inst-1");

    manager.create_lease(key.clone());
    assert!(manager.is_valid(&key), "租约创建后应该立即有效");
}

#[tokio::test]
async fn test_lease_invalid_after_ttl_expires() {
    let manager = LeaseManager::new(Duration::from_millis(50));
    let key = create_test_key("my-service", "inst-1");

    manager.create_lease(key.clone());
    time::sleep(Duration::from_millis(100)).await;

    assert!(!manager.is_valid(&key), "TTL 过期后租约应该无效");
}

#[tokio::test]
async fn test_lease_manager_clone_shares_state() {
    let manager1 = LeaseManager::new(Duration::from_secs(30));
    let manager2 = manager1.clone();

    let key = create_test_key("my-service", "inst-1");
    manager1.create_lease(key.clone());

    assert!(manager2.is_valid(&key), "克隆的 manager 应该共享状态");
    assert_eq!(manager2.count(), 1, "克隆的 manager 应该看到相同的租约数量");
}

#[tokio::test]
async fn test_eviction_callback_receives_correct_key() {
    let manager = LeaseManager::new(Duration::from_millis(100));
    let evicted_keys = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    let key1 = create_test_key("service-1", "inst-1");
    let key2 = create_test_key("service-2", "inst-2");

    manager.create_lease(key1.clone());
    manager.create_lease(key2.clone());

    // 启动清理任务
    let keys = evicted_keys.clone();
    manager.clone().start_eviction_task(
        Duration::from_millis(50),
        move |key| {
            let keys = keys.clone();
            tokio::spawn(async move {
                keys.lock().await.push(key);
            });
        },
    );

    // 等待清理
    time::sleep(Duration::from_millis(200)).await;

    let evicted = evicted_keys.lock().await;
    assert_eq!(evicted.len(), 2, "应该清理2个租约");
    assert!(evicted.contains(&key1), "应该清理 key1");
    assert!(evicted.contains(&key2), "应该清理 key2");
}
