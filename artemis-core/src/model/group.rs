use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

/// 服务分组核心结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceGroup {
    /// 分组 ID (自动生成)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<i64>,
    /// 所属服务 ID
    pub service_id: String,
    /// Region ID
    pub region_id: String,
    /// Zone ID
    pub zone_id: String,
    /// 分组名称
    pub name: String,
    /// 分组类型 (Physical/Logical)
    pub group_type: GroupType,
    /// 分组状态 (Active/Inactive)
    pub status: GroupStatus,
    /// 分组描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 分组标签
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<GroupTag>>,
    /// 分组元数据
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// 创建时间 (Unix timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
    /// 更新时间 (Unix timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<i64>,
}

/// 分组状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GroupStatus {
    /// 激活状态
    Active,
    /// 非激活状态
    Inactive,
}

/// 分组类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GroupType {
    /// 物理分组 (基于机器/IP)
    Physical,
    /// 逻辑分组 (基于标签/规则)
    Logical,
}

/// 分组标签
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupTag {
    /// 标签键
    pub key: String,
    /// 标签值
    pub value: String,
}

/// 分组实例关联
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupInstance {
    /// 关联 ID (自动生成)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    /// 分组 ID
    pub group_id: i64,
    /// 实例 ID
    pub instance_id: String,
    /// Region ID
    pub region_id: String,
    /// Zone ID
    pub zone_id: String,
    /// Service ID
    pub service_id: String,
    /// 绑定类型 (manual | auto)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub binding_type: Option<BindingType>,
    /// 操作人 ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_id: Option<String>,
    /// 创建时间 (Unix timestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
}

/// 绑定类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BindingType {
    /// 手动绑定 (通过 API 添加)
    Manual,
    /// 自动绑定 (通过 metadata 匹配)
    Auto,
}

/// 分组操作记录
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupOperation {
    /// 操作 ID (自动生成)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_id: Option<i64>,
    /// 分组 ID
    pub group_id: i64,
    /// 操作类型 (create/update/delete/activate/deactivate)
    pub operation_type: String,
    /// 操作人 ID
    pub operator_id: String,
    /// 操作描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 操作时间 (Unix timestamp)
    pub timestamp: i64,
}

impl ServiceGroup {
    /// 生成分组唯一键: service_id:region_id:zone_id:name
    pub fn group_key(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.service_id, self.region_id, self.zone_id, self.name
        )
    }
}

impl fmt::Display for GroupStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GroupStatus::Active => write!(f, "active"),
            GroupStatus::Inactive => write!(f, "inactive"),
        }
    }
}

impl FromStr for GroupStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(GroupStatus::Active),
            "inactive" => Ok(GroupStatus::Inactive),
            _ => Err(format!("Invalid GroupStatus: {}", s)),
        }
    }
}

impl fmt::Display for GroupType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GroupType::Physical => write!(f, "physical"),
            GroupType::Logical => write!(f, "logical"),
        }
    }
}

impl FromStr for GroupType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "physical" => Ok(GroupType::Physical),
            "logical" => Ok(GroupType::Logical),
            _ => Err(format!("Invalid GroupType: {}", s)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_group_serde() {
        let group = ServiceGroup {
            group_id: Some(1),
            service_id: "test-service".to_string(),
            region_id: "us-east".to_string(),
            zone_id: "zone-1".to_string(),
            name: "group-a".to_string(),
            group_type: GroupType::Physical,
            status: GroupStatus::Active,
            description: Some("Test group".to_string()),
            tags: Some(vec![GroupTag {
                key: "env".to_string(),
                value: "prod".to_string(),
            }]),
            metadata: None,
            created_at: Some(1234567890),
            updated_at: Some(1234567890),
        };

        let json = serde_json::to_string(&group).unwrap();
        let deserialized: ServiceGroup = serde_json::from_str(&json).unwrap();
        assert_eq!(group, deserialized);
    }

    #[test]
    fn test_group_status_display_and_fromstr() {
        // Test Display
        assert_eq!(GroupStatus::Active.to_string(), "active");
        assert_eq!(GroupStatus::Inactive.to_string(), "inactive");

        // Test FromStr
        assert_eq!("active".parse::<GroupStatus>().unwrap(), GroupStatus::Active);
        assert_eq!("ACTIVE".parse::<GroupStatus>().unwrap(), GroupStatus::Active);
        assert_eq!("inactive".parse::<GroupStatus>().unwrap(), GroupStatus::Inactive);
        assert!("invalid".parse::<GroupStatus>().is_err());
    }

    #[test]
    fn test_group_type_display_and_fromstr() {
        // Test Display
        assert_eq!(GroupType::Physical.to_string(), "physical");
        assert_eq!(GroupType::Logical.to_string(), "logical");

        // Test FromStr
        assert_eq!("physical".parse::<GroupType>().unwrap(), GroupType::Physical);
        assert_eq!("PHYSICAL".parse::<GroupType>().unwrap(), GroupType::Physical);
        assert_eq!("logical".parse::<GroupType>().unwrap(), GroupType::Logical);
        assert!("invalid".parse::<GroupType>().is_err());
    }
}
