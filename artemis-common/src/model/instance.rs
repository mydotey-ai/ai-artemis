use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Instance {
    pub region_id: String,
    pub zone_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    pub service_id: String,
    pub instance_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub machine_name: Option<String>,
    pub ip: String,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocol: Option<String>,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_check_url: Option<String>,
    pub status: InstanceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum InstanceStatus {
    Starting,
    Up,
    Down,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceKey {
    pub region_id: String,
    pub zone_id: String,
    pub service_id: String,
    pub group_id: String,
    pub instance_id: String,
}

impl Instance {
    pub fn key(&self) -> InstanceKey {
        InstanceKey {
            region_id: self.region_id.clone(),
            zone_id: self.zone_id.clone(),
            service_id: self.service_id.to_lowercase(),
            group_id: self.group_id.clone().unwrap_or_default(),
            instance_id: self.instance_id.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_key_generation() {
        let instance = Instance {
            region_id: "us-east".to_string(),
            zone_id: "zone-1".to_string(),
            group_id: Some("group-a".to_string()),
            service_id: "MyService".to_string(),
            instance_id: "inst-1".to_string(),
            machine_name: None,
            ip: "192.168.1.1".to_string(),
            port: 8080,
            protocol: Some("http".to_string()),
            url: "http://192.168.1.1:8080".to_string(),
            health_check_url: None,
            status: InstanceStatus::Up,
            metadata: None,
        };

        let key = instance.key();
        assert_eq!(key.service_id, "myservice"); // 转小写
        assert_eq!(key.group_id, "group-a");
    }

    #[test]
    fn test_instance_serde() {
        let instance = Instance {
            region_id: "us-east".to_string(),
            zone_id: "zone-1".to_string(),
            group_id: None,
            service_id: "test-service".to_string(),
            instance_id: "inst-1".to_string(),
            machine_name: None,
            ip: "192.168.1.1".to_string(),
            port: 8080,
            protocol: None,
            url: "http://192.168.1.1:8080".to_string(),
            health_check_url: None,
            status: InstanceStatus::Up,
            metadata: None,
        };

        let json = serde_json::to_string(&instance).unwrap();
        let deserialized: Instance = serde_json::from_str(&json).unwrap();
        assert_eq!(instance, deserialized);
    }
}
