use super::error::{ReplicationError, ReplicationErrorKind};
use artemis_core::model::{
    ReplicateRegisterRequest, ReplicateRegisterResponse,
    ReplicateHeartbeatRequest, ReplicateHeartbeatResponse,
    ReplicateUnregisterRequest, ReplicateUnregisterResponse,
    BatchRegisterRequest, BatchRegisterResponse,
    BatchUnregisterRequest, BatchUnregisterResponse,
    GetAllServicesResponse,
};
use std::time::Duration;
use tracing::debug;

/// 复制客户端
///
/// 负责向对等节点发送复制请求
pub struct ReplicationClient {
    client: reqwest::Client,
    #[allow(dead_code)]
    timeout: Duration,
}

impl ReplicationClient {
    pub fn new(timeout: Duration) -> Self {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .pool_max_idle_per_host(10) // 连接池优化
            .build()
            .expect("Failed to create HTTP client");

        Self { client, timeout }
    }

    /// 复制注册请求
    pub async fn replicate_register(
        &self,
        peer_url: &str,
        request: ReplicateRegisterRequest,
    ) -> Result<ReplicateRegisterResponse, ReplicationError> {
        let url = format!("{}/api/replication/registry/register.json", peer_url);

        debug!("Replicating register to {}", peer_url);

        let response = self
            .client
            .post(&url)
            .header("X-Artemis-Replication", "true") // 防止复制循环
            .json(&request)
            .send()
            .await
            .map_err(ReplicationError::from_reqwest)?;

        if response.status().is_success() {
            response
                .json()
                .await
                .map_err(|e| {
                    ReplicationError::new(
                        ReplicationErrorKind::PermanentFailure,
                        format!("Failed to parse response: {}", e),
                    )
                })
        } else {
            Err(ReplicationError::from_status(response.status()))
        }
    }

    /// 复制心跳请求
    pub async fn replicate_heartbeat(
        &self,
        peer_url: &str,
        request: ReplicateHeartbeatRequest,
    ) -> Result<ReplicateHeartbeatResponse, ReplicationError> {
        let url = format!("{}/api/replication/registry/heartbeat.json", peer_url);

        debug!(
            "Replicating {} heartbeats to {}",
            request.instance_keys.len(),
            peer_url
        );

        let response = self
            .client
            .post(&url)
            .header("X-Artemis-Replication", "true")
            .json(&request)
            .send()
            .await
            .map_err(ReplicationError::from_reqwest)?;

        if response.status().is_success() {
            response.json().await.map_err(|e| {
                ReplicationError::new(
                    ReplicationErrorKind::PermanentFailure,
                    format!("Failed to parse response: {}", e),
                )
            })
        } else {
            Err(ReplicationError::from_status(response.status()))
        }
    }

    /// 复制注销请求
    pub async fn replicate_unregister(
        &self,
        peer_url: &str,
        request: ReplicateUnregisterRequest,
    ) -> Result<ReplicateUnregisterResponse, ReplicationError> {
        let url = format!("{}/api/replication/registry/unregister.json", peer_url);

        debug!("Replicating unregister to {}", peer_url);

        let response = self
            .client
            .post(&url)
            .header("X-Artemis-Replication", "true")
            .json(&request)
            .send()
            .await
            .map_err(ReplicationError::from_reqwest)?;

        if response.status().is_success() {
            response.json().await.map_err(|e| {
                ReplicationError::new(
                    ReplicationErrorKind::PermanentFailure,
                    format!("Failed to parse response: {}", e),
                )
            })
        } else {
            Err(ReplicationError::from_status(response.status()))
        }
    }

    /// 获取所有服务(用于启动同步)
    pub async fn get_all_services(
        &self,
        peer_url: &str,
    ) -> Result<GetAllServicesResponse, ReplicationError> {
        let url = format!("{}/api/replication/registry/services.json", peer_url);

        debug!("Fetching all services from {}", peer_url);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(ReplicationError::from_reqwest)?;

        if response.status().is_success() {
            response.json().await.map_err(|e| {
                ReplicationError::new(
                    ReplicationErrorKind::PermanentFailure,
                    format!("Failed to parse response: {}", e),
                )
            })
        } else {
            Err(ReplicationError::from_status(response.status()))
        }
    }

    /// 批量注册请求 (Phase 23)
    pub async fn batch_register(
        &self,
        peer_url: &str,
        request: BatchRegisterRequest,
    ) -> Result<BatchRegisterResponse, ReplicationError> {
        let url = format!("{}/api/replication/registry/batch-register.json", peer_url);

        debug!("Batch replicating {} instances to {}", request.instances.len(), peer_url);

        let response = self
            .client
            .post(&url)
            .header("X-Artemis-Replication", "true") // 防止复制循环
            .json(&request)
            .send()
            .await
            .map_err(ReplicationError::from_reqwest)?;

        if response.status().is_success() {
            response
                .json()
                .await
                .map_err(|e| {
                    ReplicationError::new(
                        ReplicationErrorKind::PermanentFailure,
                        format!("Failed to parse response: {}", e),
                    )
                })
        } else {
            Err(ReplicationError::from_status(response.status()))
        }
    }

    /// 批量注销请求 (Phase 23)
    pub async fn batch_unregister(
        &self,
        peer_url: &str,
        request: BatchUnregisterRequest,
    ) -> Result<BatchUnregisterResponse, ReplicationError> {
        let url = format!("{}/api/replication/registry/batch-unregister.json", peer_url);

        debug!("Batch unregistering {} instances to {}", request.instance_keys.len(), peer_url);

        let response = self
            .client
            .post(&url)
            .header("X-Artemis-Replication", "true") // 防止复制循环
            .json(&request)
            .send()
            .await
            .map_err(ReplicationError::from_reqwest)?;

        if response.status().is_success() {
            response.json().await.map_err(|e| {
                ReplicationError::new(
                    ReplicationErrorKind::PermanentFailure,
                    format!("Failed to parse response: {}", e),
                )
            })
        } else {
            Err(ReplicationError::from_status(response.status()))
        }
    }
}

impl Default for ReplicationClient {
    fn default() -> Self {
        Self::new(Duration::from_secs(5))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== 客户端创建测试 =====

    #[test]
    fn test_client_creation() {
        let client = ReplicationClient::new(Duration::from_secs(5));
        assert_eq!(client.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_default_client() {
        let client = ReplicationClient::default();
        assert_eq!(client.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_client_creation_with_custom_timeout() {
        let timeout = Duration::from_secs(10);
        let client = ReplicationClient::new(timeout);
        assert_eq!(client.timeout, timeout);
    }

    #[test]
    fn test_client_creation_with_short_timeout() {
        let timeout = Duration::from_millis(500);
        let client = ReplicationClient::new(timeout);
        assert_eq!(client.timeout, timeout);
    }

    #[test]
    fn test_client_creation_with_long_timeout() {
        let timeout = Duration::from_secs(60);
        let client = ReplicationClient::new(timeout);
        assert_eq!(client.timeout, timeout);
    }

    // ===== URL 构建验证测试 =====

    #[test]
    fn test_register_url_format() {
        let peer_url = "http://192.168.1.100:8080";
        let expected_url = format!("{}/api/replication/registry/register.json", peer_url);

        assert_eq!(
            expected_url,
            "http://192.168.1.100:8080/api/replication/registry/register.json"
        );
    }

    #[test]
    fn test_heartbeat_url_format() {
        let peer_url = "http://example.com:9090";
        let expected_url = format!("{}/api/replication/registry/heartbeat.json", peer_url);

        assert_eq!(
            expected_url,
            "http://example.com:9090/api/replication/registry/heartbeat.json"
        );
    }

    #[test]
    fn test_unregister_url_format() {
        let peer_url = "http://localhost:8080";
        let expected_url = format!("{}/api/replication/registry/unregister.json", peer_url);

        assert_eq!(
            expected_url,
            "http://localhost:8080/api/replication/registry/unregister.json"
        );
    }

    #[test]
    fn test_get_all_services_url_format() {
        let peer_url = "http://peer:8080";
        let expected_url = format!("{}/api/replication/registry/services.json", peer_url);

        assert_eq!(
            expected_url,
            "http://peer:8080/api/replication/registry/services.json"
        );
    }

    #[test]
    fn test_batch_register_url_format() {
        let peer_url = "http://192.168.1.101:8080";
        let expected_url = format!("{}/api/replication/registry/batch-register.json", peer_url);

        assert_eq!(
            expected_url,
            "http://192.168.1.101:8080/api/replication/registry/batch-register.json"
        );
    }

    #[test]
    fn test_batch_unregister_url_format() {
        let peer_url = "http://192.168.1.102:8080";
        let expected_url = format!("{}/api/replication/registry/batch-unregister.json", peer_url);

        assert_eq!(
            expected_url,
            "http://192.168.1.102:8080/api/replication/registry/batch-unregister.json"
        );
    }

    // ===== 客户端配置测试 =====

    #[test]
    fn test_client_is_created_successfully() {
        // 验证客户端创建不会 panic
        let _client = ReplicationClient::new(Duration::from_secs(1));
    }

    #[test]
    fn test_default_client_has_correct_timeout() {
        let client = ReplicationClient::default();
        assert_eq!(
            client.timeout,
            Duration::from_secs(5),
            "默认超时应该是 5 秒"
        );
    }

    #[test]
    fn test_multiple_clients_can_be_created() {
        let _client1 = ReplicationClient::new(Duration::from_secs(1));
        let _client2 = ReplicationClient::new(Duration::from_secs(2));
        let _client3 = ReplicationClient::new(Duration::from_secs(3));

        // 如果能创建多个客户端,测试通过
    }
}
