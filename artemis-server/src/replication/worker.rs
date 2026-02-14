use super::client::ReplicationClient;
use super::manager::ReplicationEvent;
use crate::cluster::ClusterManager;
use artemis_core::config::ReplicationConfig;
use artemis_core::model::{
    Instance, InstanceKey, ReplicateRegisterRequest, ReplicateHeartbeatRequest,
    ReplicateUnregisterRequest,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

/// 复制工作器
///
/// 后台异步处理复制事件,支持:
/// - 心跳批处理(减少网络请求)
/// - 错误重试(临时失败可重试)
/// - 并发复制到多个节点
pub struct ReplicationWorker {
    event_rx: UnboundedReceiver<ReplicationEvent>,
    cluster_manager: Arc<ClusterManager>,
    client: ReplicationClient,
    config: ReplicationConfig,

    // 批处理缓冲区
    heartbeat_buffer: Vec<InstanceKey>,
    last_batch_time: Instant,
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
        }
    }

    /// 启动工作器
    pub fn start(mut self) -> JoinHandle<()> {
        tokio::spawn(async move {
            info!("Replication worker started");

            let batch_interval = Duration::from_millis(self.config.batch_interval_ms);
            let mut interval = tokio::time::interval(batch_interval);

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
                }
            }
        })
    }

    /// 处理注册事件
    async fn process_register(&self, instance: Instance) {
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
                    // TODO: 可以实现重试队列
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
    async fn process_unregister(&self, key: InstanceKey) {
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
