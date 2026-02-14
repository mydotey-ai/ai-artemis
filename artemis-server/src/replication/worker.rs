use super::client::ReplicationClient;
use super::manager::ReplicationEvent;
use crate::cluster::ClusterManager;
use artemis_core::config::ReplicationConfig;
use artemis_core::model::{
    Instance, InstanceKey, ReplicateRegisterRequest, ReplicateHeartbeatRequest,
    ReplicateUnregisterRequest,
};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

/// 重试项
#[derive(Debug, Clone)]
struct RetryItem {
    /// 目标节点ID
    node_id: String,
    /// 复制事件
    event: ReplicationEvent,
    /// 重试次数
    retry_count: u32,
    /// 下次重试时间
    next_retry_time: Instant,
}

/// 复制工作器
///
/// 后台异步处理复制事件,支持:
/// - 心跳批处理(减少网络请求)
/// - 智能重试队列(临时失败自动重试)
/// - 并发复制到多个节点
pub struct ReplicationWorker {
    event_rx: UnboundedReceiver<ReplicationEvent>,
    cluster_manager: Arc<ClusterManager>,
    client: ReplicationClient,
    config: ReplicationConfig,

    // 批处理缓冲区
    heartbeat_buffer: Vec<InstanceKey>,
    last_batch_time: Instant,

    // 重试队列
    retry_queue: VecDeque<RetryItem>,
}

impl ReplicationWorker {
    pub fn new(
        event_rx: UnboundedReceiver<ReplicationEvent>,
        cluster_manager: Arc<ClusterManager>,
        config: ReplicationConfig,
    ) -> Self {
        let timeout = Duration::from_secs(config.timeout_secs);
        let client = ReplicationClient::new(timeout);

        Self {
            event_rx,
            cluster_manager,
            client,
            config,
            heartbeat_buffer: Vec::new(),
            last_batch_time: Instant::now(),
            retry_queue: VecDeque::new(),
        }
    }

    /// 启动工作器
    pub fn start(mut self) -> JoinHandle<()> {
        tokio::spawn(async move {
            info!("Replication worker started with retry queue");

            let batch_interval = Duration::from_millis(self.config.batch_interval_ms);
            let mut interval = tokio::time::interval(batch_interval);

            // 重试检查间隔 (1秒)
            let retry_interval = Duration::from_secs(1);
            let mut retry_timer = tokio::time::interval(retry_interval);

            loop {
                tokio::select! {
                    // 处理新事件
                    Some(event) = self.event_rx.recv() => {
                        match event {
                            ReplicationEvent::Register(instance) => {
                                self.process_register(instance).await;
                            }
                            ReplicationEvent::Heartbeat(key) => {
                                self.heartbeat_buffer.push(key);
                            }
                            ReplicationEvent::Unregister(key) => {
                                self.process_unregister(key).await;
                            }
                        }
                    }

                    // 定期刷新批处理
                    _ = interval.tick() => {
                        if !self.heartbeat_buffer.is_empty() {
                            self.flush_heartbeat_batch().await;
                        }
                    }

                    // 定期处理重试队列
                    _ = retry_timer.tick() => {
                        self.process_retry_queue().await;
                    }
                }
            }
        })
    }

    /// 处理注册事件
    async fn process_register(&mut self, instance: Instance) {
        let peers = self.cluster_manager.get_healthy_peers();

        if peers.is_empty() {
            debug!("No healthy peers to replicate register");
            return;
        }

        info!(
            "Replicating register for {} to {} peers",
            instance.instance_id,
            peers.len()
        );

        for peer in peers {
            let request = ReplicateRegisterRequest {
                instances: vec![instance.clone()],
            };

            match self
                .client
                .replicate_register(&peer.base_url(), request)
                .await
            {
                Ok(_) => {
                    debug!("Successfully replicated register to {}", peer.node_id);
                }
                Err(e) if e.is_retryable() => {
                    warn!(
                        "Retryable error replicating register to {}: {}",
                        peer.node_id, e
                    );
                    // 添加到重试队列
                    self.add_to_retry_queue(
                        peer.node_id.clone(),
                        ReplicationEvent::Register(instance.clone()),
                        0,
                    );
                }
                Err(e) => {
                    warn!(
                        "Permanent error replicating register to {}: {}",
                        peer.node_id, e
                    );
                }
            }
        }
    }

    /// 处理注销事件
    async fn process_unregister(&mut self, key: InstanceKey) {
        let peers = self.cluster_manager.get_healthy_peers();

        if peers.is_empty() {
            debug!("No healthy peers to replicate unregister");
            return;
        }

        info!(
            "Replicating unregister for {} to {} peers",
            key.instance_id,
            peers.len()
        );

        for peer in peers {
            let request = ReplicateUnregisterRequest {
                instance_keys: vec![key.clone()],
            };

            match self
                .client
                .replicate_unregister(&peer.base_url(), request)
                .await
            {
                Ok(_) => {
                    debug!("Successfully replicated unregister to {}", peer.node_id);
                }
                Err(e) if e.is_retryable() => {
                    warn!(
                        "Retryable error replicating unregister to {}: {}",
                        peer.node_id, e
                    );
                    // 添加到重试队列
                    self.add_to_retry_queue(
                        peer.node_id.clone(),
                        ReplicationEvent::Unregister(key.clone()),
                        0,
                    );
                }
                Err(e) => {
                    warn!(
                        "Permanent error replicating unregister to {}: {}",
                        peer.node_id, e
                    );
                }
            }
        }
    }

    /// 刷新心跳批处理
    async fn flush_heartbeat_batch(&mut self) {
        if self.heartbeat_buffer.is_empty() {
            return;
        }

        let keys = std::mem::take(&mut self.heartbeat_buffer);
        let peers = self.cluster_manager.get_healthy_peers();

        if peers.is_empty() {
            debug!("No healthy peers to replicate heartbeats");
            return;
        }

        info!(
            "Replicating {} heartbeats to {} peers",
            keys.len(),
            peers.len()
        );

        for peer in peers {
            let request = ReplicateHeartbeatRequest {
                instance_keys: keys.clone(),
            };

            match self
                .client
                .replicate_heartbeat(&peer.base_url(), request)
                .await
            {
                Ok(_) => {
                    debug!(
                        "Successfully replicated {} heartbeats to {}",
                        keys.len(),
                        peer.node_id
                    );
                }
                Err(e) if e.is_retryable() => {
                    warn!(
                        "Retryable error replicating heartbeats to {}: {}",
                        peer.node_id, e
                    );
                    // 心跳批处理失败,将每个心跳单独加入重试队列
                    for key in &keys {
                        self.add_to_retry_queue(
                            peer.node_id.clone(),
                            ReplicationEvent::Heartbeat(key.clone()),
                            0,
                        );
                    }
                }
                Err(e) => {
                    warn!(
                        "Permanent error replicating heartbeats to {}: {}",
                        peer.node_id, e
                    );
                }
            }
        }

        self.last_batch_time = Instant::now();
    }

    /// 添加项到重试队列
    fn add_to_retry_queue(&mut self, node_id: String, event: ReplicationEvent, retry_count: u32) {
        // 检查是否超过最大重试次数
        if retry_count >= self.config.max_retries {
            warn!(
                "Max retries ({}) exceeded for event to {}, dropping",
                self.config.max_retries, node_id
            );
            return;
        }

        // 使用指数退避策略: 2^retry_count 秒
        let backoff_secs = 2u64.pow(retry_count);
        let next_retry_time = Instant::now() + Duration::from_secs(backoff_secs);

        let item = RetryItem {
            node_id: node_id.clone(),
            event,
            retry_count,
            next_retry_time,
        };

        self.retry_queue.push_back(item);

        debug!(
            "Added event to retry queue for {}, retry {} of {}, next retry in {}s",
            node_id, retry_count + 1, self.config.max_retries, backoff_secs
        );
    }

    /// 处理重试队列
    async fn process_retry_queue(&mut self) {
        let now = Instant::now();
        let mut items_to_retry = Vec::new();

        // 收集需要重试的项
        while let Some(item) = self.retry_queue.front() {
            if item.next_retry_time <= now {
                items_to_retry.push(self.retry_queue.pop_front().unwrap());
            } else {
                break; // 队列是按时间排序的
            }
        }

        if items_to_retry.is_empty() {
            return;
        }

        debug!("Processing {} items from retry queue", items_to_retry.len());

        // 重试每个项
        for item in items_to_retry {
            self.retry_event(item).await;
        }
    }

    /// 重试单个事件
    async fn retry_event(&mut self, item: RetryItem) {
        let RetryItem {
            node_id,
            event,
            retry_count,
            ..
        } = item;

        // 获取节点信息
        let peer = match self
            .cluster_manager
            .get_healthy_peers()
            .into_iter()
            .find(|p| p.node_id == node_id)
        {
            Some(p) => p,
            None => {
                warn!("Node {} not found or unhealthy, dropping retry", node_id);
                return;
            }
        };

        // 根据事件类型执行重试并处理结果
        match event {
            ReplicationEvent::Register(instance) => {
                let request = ReplicateRegisterRequest {
                    instances: vec![instance.clone()],
                };
                match self
                    .client
                    .replicate_register(&peer.base_url(), request)
                    .await
                {
                    Ok(_) => {
                        info!(
                            "Successfully retried register to {} (attempt {})",
                            node_id,
                            retry_count + 1
                        );
                    }
                    Err(e) if e.is_retryable() => {
                        warn!(
                            "Retry attempt {} failed for {}: {}",
                            retry_count + 1,
                            node_id,
                            e
                        );
                        self.add_to_retry_queue(
                            node_id,
                            ReplicationEvent::Register(instance),
                            retry_count + 1,
                        );
                    }
                    Err(e) => {
                        warn!(
                            "Permanent error on retry to {}: {}, dropping",
                            node_id, e
                        );
                    }
                }
            }
            ReplicationEvent::Heartbeat(key) => {
                let request = ReplicateHeartbeatRequest {
                    instance_keys: vec![key.clone()],
                };
                match self
                    .client
                    .replicate_heartbeat(&peer.base_url(), request)
                    .await
                {
                    Ok(_) => {
                        info!(
                            "Successfully retried heartbeat to {} (attempt {})",
                            node_id,
                            retry_count + 1
                        );
                    }
                    Err(e) if e.is_retryable() => {
                        warn!(
                            "Retry attempt {} failed for {}: {}",
                            retry_count + 1,
                            node_id,
                            e
                        );
                        self.add_to_retry_queue(
                            node_id,
                            ReplicationEvent::Heartbeat(key),
                            retry_count + 1,
                        );
                    }
                    Err(e) => {
                        warn!(
                            "Permanent error on retry to {}: {}, dropping",
                            node_id, e
                        );
                    }
                }
            }
            ReplicationEvent::Unregister(key) => {
                let request = ReplicateUnregisterRequest {
                    instance_keys: vec![key.clone()],
                };
                match self
                    .client
                    .replicate_unregister(&peer.base_url(), request)
                    .await
                {
                    Ok(_) => {
                        info!(
                            "Successfully retried unregister to {} (attempt {})",
                            node_id,
                            retry_count + 1
                        );
                    }
                    Err(e) if e.is_retryable() => {
                        warn!(
                            "Retry attempt {} failed for {}: {}",
                            retry_count + 1,
                            node_id,
                            e
                        );
                        self.add_to_retry_queue(
                            node_id,
                            ReplicationEvent::Unregister(key),
                            retry_count + 1,
                        );
                    }
                    Err(e) => {
                        warn!(
                            "Permanent error on retry to {}: {}, dropping",
                            node_id, e
                        );
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use artemis_core::model::InstanceStatus;

    fn create_test_config() -> ReplicationConfig {
        ReplicationConfig {
            enabled: true,
            timeout_secs: 5,
            batch_size: 100,
            batch_interval_ms: 100,
            max_retries: 3,
        }
    }

    fn create_test_instance() -> Instance {
        Instance {
            region_id: "test".to_string(),
            zone_id: "zone".to_string(),
            service_id: "service".to_string(),
            group_id: None,
            instance_id: "inst-1".to_string(),
            machine_name: None,
            ip: "127.0.0.1".to_string(),
            port: 8080,
            protocol: None,
            url: "http://127.0.0.1:8080".to_string(),
            health_check_url: None,
            status: InstanceStatus::Up,
            metadata: None,
        }
    }

    #[test]
    fn test_worker_creation() {
        let (_, event_rx) = tokio::sync::mpsc::unbounded_channel();
        let cluster_manager = Arc::new(ClusterManager::default());
        let config = create_test_config();

        let worker = ReplicationWorker::new(event_rx, cluster_manager, config);
        assert_eq!(worker.heartbeat_buffer.len(), 0);
    }
}
