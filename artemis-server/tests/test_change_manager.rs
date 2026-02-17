//! InstanceChangeManager 测试
//!
//! 测试覆盖:
//! - 订阅和发布机制
//! - 不同类型的变更事件 (New/Delete/Change)
//! - 多订阅者场景
//! - 并发订阅和发布
//! - 边界条件和异常场景

use artemis_core::model::{ChangeType, Instance, InstanceKey, InstanceStatus};
use artemis_server::change::InstanceChangeManager;
use std::sync::Arc;
use tokio::time::{Duration, timeout};

/// 创建测试实例
fn create_test_instance(service_id: &str, instance_id: &str, status: InstanceStatus) -> Instance {
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
        url: format!("http://192.168.1.100:8080/{}", instance_id),
        health_check_url: None,
        status,
        metadata: None,
    }
}

/// 创建测试 InstanceKey
fn create_test_key(service_id: &str, instance_id: &str) -> InstanceKey {
    InstanceKey {
        region_id: "test-region".to_string(),
        zone_id: "test-zone".to_string(),
        service_id: service_id.to_string(),
        group_id: "default".to_string(),
        instance_id: instance_id.to_string(),
    }
}

// ===== 订阅和发布机制测试 =====

#[tokio::test]
async fn test_subscribe_creates_channel() {
    let manager = InstanceChangeManager::new();

    assert_eq!(manager.subscription_count(), 0, "初始订阅数应为 0");

    let _rx = manager.subscribe("my-service");

    assert_eq!(manager.subscription_count(), 1, "订阅后数量应为 1");
}

#[tokio::test]
async fn test_publish_register_sends_new_change() {
    let manager = InstanceChangeManager::new();
    let mut rx = manager.subscribe("my-service");

    let instance = create_test_instance("my-service", "inst-1", InstanceStatus::Up);
    manager.publish_register(&instance);

    let change = timeout(Duration::from_millis(100), rx.recv())
        .await
        .expect("接收超时")
        .expect("通道应该有消息");

    assert_eq!(change.change_type, ChangeType::New, "应该是 New 类型");
    assert_eq!(change.instance.instance_id, "inst-1");
    assert_eq!(change.instance.service_id, "my-service");
}

#[tokio::test]
async fn test_publish_unregister_sends_delete_change() {
    let manager = InstanceChangeManager::new();
    let mut rx = manager.subscribe("my-service");

    let instance = create_test_instance("my-service", "inst-1", InstanceStatus::Up);
    let key = create_test_key("my-service", "inst-1");

    manager.publish_unregister(&key, &instance);

    let change = timeout(Duration::from_millis(100), rx.recv())
        .await
        .expect("接收超时")
        .expect("通道应该有消息");

    assert_eq!(change.change_type, ChangeType::Delete, "应该是 Delete 类型");
    assert_eq!(change.instance.instance_id, "inst-1");
}

#[tokio::test]
async fn test_publish_update_sends_change() {
    let manager = InstanceChangeManager::new();
    let mut rx = manager.subscribe("my-service");

    let instance = create_test_instance("my-service", "inst-1", InstanceStatus::Down);
    manager.publish_update(&instance);

    let change = timeout(Duration::from_millis(100), rx.recv())
        .await
        .expect("接收超时")
        .expect("通道应该有消息");

    assert_eq!(change.change_type, ChangeType::Change, "应该是 Change 类型");
    assert_eq!(change.instance.status, InstanceStatus::Down);
}

#[tokio::test]
async fn test_publish_to_nonexistent_subscription_is_safe() {
    let manager = InstanceChangeManager::new();

    // 没有订阅者,发布不应该 panic
    let instance = create_test_instance("my-service", "inst-1", InstanceStatus::Up);
    manager.publish_register(&instance);

    // 测试通过,没有 panic
}

// ===== 多订阅者场景测试 =====

#[tokio::test]
async fn test_multiple_services_separate_channels() {
    let manager = InstanceChangeManager::new();

    let mut rx1 = manager.subscribe("service-1");
    let mut rx2 = manager.subscribe("service-2");

    let inst1 = create_test_instance("service-1", "inst-1", InstanceStatus::Up);
    let inst2 = create_test_instance("service-2", "inst-2", InstanceStatus::Up);

    manager.publish_register(&inst1);
    manager.publish_register(&inst2);

    // service-1 的订阅者只收到 service-1 的变更
    let change1 = timeout(Duration::from_millis(100), rx1.recv())
        .await
        .expect("接收超时")
        .expect("通道应该有消息");
    assert_eq!(change1.instance.service_id, "service-1");

    // service-2 的订阅者只收到 service-2 的变更
    let change2 = timeout(Duration::from_millis(100), rx2.recv())
        .await
        .expect("接收超时")
        .expect("通道应该有消息");
    assert_eq!(change2.instance.service_id, "service-2");
}

#[tokio::test]
async fn test_resubscribe_replaces_old_subscription() {
    let manager = InstanceChangeManager::new();

    let _rx1 = manager.subscribe("my-service");

    // 再次订阅同一个服务,会替换旧的发送者
    let mut rx2 = manager.subscribe("my-service");

    // 发布到旧订阅者不再接收的通道
    let instance1 = create_test_instance("my-service", "inst-1", InstanceStatus::Up);
    manager.publish_register(&instance1);

    // 新订阅者 rx2 应该收到第一条消息
    let change2 = timeout(Duration::from_millis(100), rx2.recv())
        .await
        .expect("接收超时")
        .expect("新订阅者应该收到消息");
    assert_eq!(change2.instance.instance_id, "inst-1");

    // 再发布一条消息
    let instance2 = create_test_instance("my-service", "inst-2", InstanceStatus::Up);
    manager.publish_register(&instance2);

    // 只有新订阅者能收到后续消息
    let change3 = timeout(Duration::from_millis(100), rx2.recv())
        .await
        .expect("接收超时")
        .expect("新订阅者应该收到第二条消息");
    assert_eq!(change3.instance.instance_id, "inst-2");

    // 订阅数量仍然是 1 (被替换了)
    assert_eq!(manager.subscription_count(), 1);
}

#[tokio::test]
async fn test_multiple_changes_received_in_order() {
    let manager = InstanceChangeManager::new();
    let mut rx = manager.subscribe("my-service");

    let inst1 = create_test_instance("my-service", "inst-1", InstanceStatus::Up);
    let inst2 = create_test_instance("my-service", "inst-2", InstanceStatus::Up);
    let inst3 = create_test_instance("my-service", "inst-3", InstanceStatus::Up);

    manager.publish_register(&inst1);
    manager.publish_register(&inst2);
    manager.publish_register(&inst3);

    // 按顺序接收变更
    let change1 = rx.recv().await.unwrap();
    assert_eq!(change1.instance.instance_id, "inst-1");

    let change2 = rx.recv().await.unwrap();
    assert_eq!(change2.instance.instance_id, "inst-2");

    let change3 = rx.recv().await.unwrap();
    assert_eq!(change3.instance.instance_id, "inst-3");
}

// ===== 并发订阅和发布测试 =====

#[tokio::test]
async fn test_concurrent_subscriptions() {
    let manager = Arc::new(InstanceChangeManager::new());
    let mut handles = vec![];

    // 10 个并发任务订阅不同服务
    for i in 0..10 {
        let mgr = manager.clone();
        let handle = tokio::spawn(async move {
            let _rx = mgr.subscribe(&format!("service-{}", i));
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    assert_eq!(manager.subscription_count(), 10, "应该有 10 个订阅");
}

#[tokio::test]
async fn test_concurrent_publish() {
    let manager = Arc::new(InstanceChangeManager::new());
    let mut rx = manager.subscribe("my-service");

    let mgr = manager.clone();

    // 并发发布 10 个变更
    let publish_task = tokio::spawn(async move {
        for i in 0..10 {
            let instance =
                create_test_instance("my-service", &format!("inst-{}", i), InstanceStatus::Up);
            mgr.publish_register(&instance);
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
    });

    // 接收 10 个变更
    let mut received_count = 0;
    while received_count < 10 {
        if timeout(Duration::from_millis(100), rx.recv()).await.is_ok() {
            received_count += 1;
        } else {
            break;
        }
    }

    publish_task.await.unwrap();
    assert_eq!(received_count, 10, "应该接收到 10 个变更");
}

#[tokio::test]
async fn test_concurrent_subscribe_and_publish() {
    let manager = Arc::new(InstanceChangeManager::new());
    let mut subscribe_handles = vec![];
    let mut publish_handles = vec![];

    // 5 个订阅任务
    for i in 0..5 {
        let mgr = manager.clone();
        let handle = tokio::spawn(async move {
            let mut rx = mgr.subscribe(&format!("service-{}", i));

            // 等待一个变更
            timeout(Duration::from_secs(1), rx.recv()).await
        });
        subscribe_handles.push(handle);
    }

    // 5 个发布任务
    for i in 0..5 {
        let mgr = manager.clone();
        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            let instance =
                create_test_instance(&format!("service-{}", i), "inst-1", InstanceStatus::Up);
            mgr.publish_register(&instance);
        });
        publish_handles.push(handle);
    }

    // 等待所有发布任务完成
    for handle in publish_handles {
        handle.await.unwrap();
    }

    // 等待所有订阅任务完成
    for handle in subscribe_handles {
        handle.await.unwrap().ok();
    }

    assert_eq!(manager.subscription_count(), 5, "应该有 5 个订阅");
}

// ===== Default 和 Clone 测试 =====

#[tokio::test]
async fn test_default_constructor() {
    let manager = InstanceChangeManager::default();

    assert_eq!(manager.subscription_count(), 0, "默认构造器应该初始化空订阅");
}

#[tokio::test]
async fn test_clone_shares_state() {
    let manager1 = InstanceChangeManager::new();
    let _rx = manager1.subscribe("my-service");

    let manager2 = manager1.clone();

    assert_eq!(manager2.subscription_count(), 1, "克隆应该共享订阅状态");

    let _rx2 = manager2.subscribe("another-service");

    assert_eq!(manager1.subscription_count(), 2, "manager1 应该看到 manager2 的订阅");
}

// ===== 边界条件和异常场景测试 =====

#[tokio::test]
async fn test_subscription_count_with_no_subscriptions() {
    let manager = InstanceChangeManager::new();

    assert_eq!(manager.subscription_count(), 0, "无订阅时应返回 0");
}

#[tokio::test]
async fn test_receiver_dropped_publish_continues() {
    let manager = InstanceChangeManager::new();

    {
        let _rx = manager.subscribe("my-service");
        // rx 在这里被 drop
    }

    // 订阅者已关闭,但发布不应该 panic
    let instance = create_test_instance("my-service", "inst-1", InstanceStatus::Up);
    manager.publish_register(&instance);

    // 测试通过,没有 panic
}

#[tokio::test]
async fn test_empty_service_id() {
    let manager = InstanceChangeManager::new();
    let mut rx = manager.subscribe("");

    let instance = create_test_instance("", "inst-1", InstanceStatus::Up);
    manager.publish_register(&instance);

    let change = timeout(Duration::from_millis(100), rx.recv())
        .await
        .expect("接收超时")
        .expect("空服务 ID 应该也能工作");

    assert_eq!(change.instance.service_id, "");
}

#[tokio::test]
async fn test_special_characters_in_service_id() {
    let manager = InstanceChangeManager::new();
    let service_id = "my-service-@#$%^&*()_+";
    let mut rx = manager.subscribe(service_id);

    let instance = create_test_instance(service_id, "inst-1", InstanceStatus::Up);
    manager.publish_register(&instance);

    let change = timeout(Duration::from_millis(100), rx.recv())
        .await
        .expect("接收超时")
        .expect("特殊字符服务 ID 应该也能工作");

    assert_eq!(change.instance.service_id, service_id);
}

#[tokio::test]
async fn test_very_long_service_id() {
    let manager = InstanceChangeManager::new();
    let service_id = "a".repeat(1000);
    let mut rx = manager.subscribe(&service_id);

    let instance = create_test_instance(&service_id, "inst-1", InstanceStatus::Up);
    manager.publish_register(&instance);

    let change = timeout(Duration::from_millis(100), rx.recv())
        .await
        .expect("接收超时")
        .expect("长服务 ID 应该也能工作");

    assert_eq!(change.instance.service_id.len(), 1000);
}

#[tokio::test]
async fn test_change_time_is_recent() {
    let manager = InstanceChangeManager::new();
    let mut rx = manager.subscribe("my-service");

    let before = chrono::Utc::now();

    let instance = create_test_instance("my-service", "inst-1", InstanceStatus::Up);
    manager.publish_register(&instance);

    let after = chrono::Utc::now();

    let change = rx.recv().await.unwrap();

    assert!(change.change_time >= before, "变更时间应该在发布之后");
    assert!(change.change_time <= after, "变更时间应该在接收之前");
}

#[tokio::test]
async fn test_all_change_types() {
    let manager = InstanceChangeManager::new();
    let mut rx = manager.subscribe("my-service");

    let instance = create_test_instance("my-service", "inst-1", InstanceStatus::Up);
    let key = create_test_key("my-service", "inst-1");

    // New
    manager.publish_register(&instance);
    let change1 = rx.recv().await.unwrap();
    assert_eq!(change1.change_type, ChangeType::New);

    // Change
    manager.publish_update(&instance);
    let change2 = rx.recv().await.unwrap();
    assert_eq!(change2.change_type, ChangeType::Change);

    // Delete
    manager.publish_unregister(&key, &instance);
    let change3 = rx.recv().await.unwrap();
    assert_eq!(change3.change_type, ChangeType::Delete);
}

#[tokio::test]
async fn test_high_throughput_publishing() {
    let manager = Arc::new(InstanceChangeManager::new());
    let mut rx = manager.subscribe("my-service");

    let mgr = manager.clone();

    // 快速发布 100 个变更
    let publish_task = tokio::spawn(async move {
        for i in 0..100 {
            let instance =
                create_test_instance("my-service", &format!("inst-{}", i), InstanceStatus::Up);
            mgr.publish_register(&instance);
        }
    });

    // 接收 100 个变更
    let mut received_count = 0;
    while received_count < 100 {
        if timeout(Duration::from_millis(1000), rx.recv()).await.is_ok() {
            received_count += 1;
        } else {
            break;
        }
    }

    publish_task.await.unwrap();
    assert_eq!(received_count, 100, "应该接收到所有 100 个变更");
}
