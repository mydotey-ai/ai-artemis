//! 路由管理 HTTP API
//!
//! 提供分组和路由规则管理的 REST API

use crate::state::AppState;
use artemis_core::model::{
    GroupStatus, GroupType, RouteRule, RouteRuleGroup, RouteRuleStatus, RouteStrategy,
    ServiceGroup,
};
use artemis_core::model::group::GroupInstance;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

// ===== 请求/响应模型 =====

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateGroupRequest {
    pub service_id: String,
    pub region_id: String,
    pub zone_id: String,
    pub name: String,
    pub group_type: GroupType,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateRuleRequest {
    pub route_id: String,
    pub service_id: String,
    pub name: String,
    pub description: Option<String>,
    pub strategy: RouteStrategy,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddRuleGroupRequest {
    pub group_id: String,
    pub weight: u32,
    pub region_id: Option<String>,
    pub zone_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListGroupsQuery {
    pub service_id: Option<String>,
    pub region_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateGroupRequest {
    pub description: Option<String>,
    pub status: Option<GroupStatus>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRuleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub strategy: Option<RouteStrategy>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateRuleGroupRequest {
    pub weight: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddGroupTagsRequest {
    pub tags: Vec<artemis_core::model::GroupTag>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetGroupInstancesQuery {
    pub region_id: Option<String>,
    pub zone_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            message: Some(message),
        }
    }
}

// ===== 分组管理 API =====

/// POST /api/routing/groups - 创建分组
pub async fn create_group(
    State(state): State<AppState>,
    Json(req): Json<CreateGroupRequest>,
) -> impl IntoResponse {
    let group = ServiceGroup {
        group_id: None,
        service_id: req.service_id,
        region_id: req.region_id,
        zone_id: req.zone_id,
        name: req.name,
        group_type: req.group_type,
        status: GroupStatus::Active,
        description: req.description,
        tags: None,
        metadata: None,
        created_at: None,
        updated_at: None,
    };

    match state.group_manager.create_group(group.clone()) {
        Ok(_) => {
            // 返回创建后的分组 (包含自动生成的 ID)
            let group_key = group.group_key();
            match state.group_manager.get_group(&group_key) {
                Some(created_group) => (StatusCode::CREATED, Json(ApiResponse::success(created_group))),
                None => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<ServiceGroup>::error("Failed to retrieve created group".to_string())),
                ),
            }
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::<ServiceGroup>::error(e))),
    }
}

/// GET /api/routing/groups/:group_id - 获取分组
pub async fn get_group(
    State(state): State<AppState>,
    Path(group_id): Path<i64>,
) -> impl IntoResponse {
    match state.group_manager.get_group_by_id(group_id) {
        Some(group) => (StatusCode::OK, Json(ApiResponse::success(group))),
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<ServiceGroup>::error(format!("Group {} not found", group_id))),
        ),
    }
}

/// GET /api/routing/groups - 列出分组
pub async fn list_groups(
    State(state): State<AppState>,
    Query(query): Query<ListGroupsQuery>,
) -> impl IntoResponse {
    let groups = if let Some(service_id) = query.service_id {
        state.group_manager.list_groups_by_service(&service_id)
    } else if let Some(region_id) = query.region_id {
        state.group_manager.list_groups_by_region(&region_id)
    } else {
        state.group_manager.list_groups()
    };

    (StatusCode::OK, Json(ApiResponse::success(groups)))
}

/// DELETE /api/routing/groups/:group_key - 删除分组
pub async fn delete_group(
    State(state): State<AppState>,
    Path(group_key): Path<String>,
) -> impl IntoResponse {
    match state.group_manager.delete_group(&group_key) {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success(())),
        ),
        Err(e) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error(e))),
    }
}

/// PATCH /api/routing/groups/:group_key - 更新分组
pub async fn update_group(
    State(state): State<AppState>,
    Path(group_key): Path<String>,
    Json(req): Json<UpdateGroupRequest>,
) -> impl IntoResponse {
    match state.group_manager.get_group(&group_key) {
        Some(mut group) => {
            if let Some(description) = req.description {
                group.description = Some(description);
            }
            if let Some(status) = req.status {
                group.status = status;
            }

            match state.group_manager.update_group(group.clone()) {
                Ok(_) => (StatusCode::OK, Json(ApiResponse::success(group))),
                Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::<ServiceGroup>::error(e))),
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<ServiceGroup>::error(format!("Group {} not found", group_key))),
        ),
    }
}

/// POST /api/routing/groups/:group_key/tags - 添加标签到分组
pub async fn add_group_tags(
    State(state): State<AppState>,
    Path(group_key): Path<String>,
    Json(req): Json<AddGroupTagsRequest>,
) -> impl IntoResponse {
    match state.group_manager.get_group(&group_key) {
        Some(mut group) => {
            let mut tags = group.tags.unwrap_or_default();
            tags.extend(req.tags);
            group.tags = Some(tags);

            match state.group_manager.update_group(group.clone()) {
                Ok(_) => (StatusCode::OK, Json(ApiResponse::success(group))),
                Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::<ServiceGroup>::error(e))),
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<ServiceGroup>::error(format!("Group {} not found", group_key))),
        ),
    }
}

/// GET /api/routing/groups/:group_key/tags - 获取分组标签
pub async fn get_group_tags(
    State(state): State<AppState>,
    Path(group_key): Path<String>,
) -> impl IntoResponse {
    match state.group_manager.get_group(&group_key) {
        Some(group) => {
            let tags = group.tags.unwrap_or_default();
            (StatusCode::OK, Json(ApiResponse::success(tags)))
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<Vec<artemis_core::model::GroupTag>>::error(
                format!("Group {} not found", group_key)
            )),
        ),
    }
}

/// DELETE /api/routing/groups/:group_key/tags/:tag_key - 删除分组标签
pub async fn remove_group_tag(
    State(state): State<AppState>,
    Path((group_key, tag_key)): Path<(String, String)>,
) -> impl IntoResponse {
    match state.group_manager.get_group(&group_key) {
        Some(mut group) => {
            if let Some(mut tags) = group.tags {
                tags.retain(|tag| tag.key != tag_key);
                group.tags = Some(tags);

                match state.group_manager.update_group(group.clone()) {
                    Ok(_) => (StatusCode::OK, Json(ApiResponse::success(group))),
                    Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::<ServiceGroup>::error(e))),
                }
            } else {
                (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::<ServiceGroup>::error("No tags found".to_string())),
                )
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<ServiceGroup>::error(format!("Group {} not found", group_key))),
        ),
    }
}

/// GET /api/routing/groups/:group_key/instances - 获取分组的实例
pub async fn get_group_instances(
    State(state): State<AppState>,
    Path(group_key): Path<String>,
    Query(query): Query<GetGroupInstancesQuery>,
) -> impl IntoResponse {
    match state.group_manager.get_group(&group_key) {
        Some(group) => {
            let instances = state
                .registry_service
                .get_instances_by_group(&group.service_id, &group.name, query.region_id.as_deref());
            (StatusCode::OK, Json(ApiResponse::success(instances)))
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<Vec<artemis_core::model::Instance>>::error(
                format!("Group {} not found", group_key)
            )),
        ),
    }
}

// ===== 路由规则 API =====

/// POST /api/routing/rules - 创建路由规则
pub async fn create_rule(
    State(state): State<AppState>,
    Json(req): Json<CreateRuleRequest>,
) -> impl IntoResponse {
    let rule = RouteRule {
        route_rule_id: None,
        route_id: req.route_id,
        service_id: req.service_id,
        name: req.name,
        description: req.description,
        status: RouteRuleStatus::Inactive, // 默认未激活
        strategy: req.strategy,
        groups: vec![],
    };

    match state.route_manager.create_rule(rule.clone()) {
        Ok(_) => {
            // 返回创建后的规则 (包含自动生成的 ID)
            match state.route_manager.get_rule(&rule.route_id) {
                Some(created_rule) => (StatusCode::CREATED, Json(ApiResponse::success(created_rule))),
                None => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<RouteRule>::error("Failed to retrieve created rule".to_string())),
                ),
            }
        }
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::<RouteRule>::error(e))),
    }
}

/// GET /api/routing/rules/:rule_id - 获取路由规则
pub async fn get_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
) -> impl IntoResponse {
    match state.route_manager.get_rule(&rule_id) {
        Some(rule) => (StatusCode::OK, Json(ApiResponse::success(rule))),
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<RouteRule>::error(format!("Rule {} not found", rule_id))),
        ),
    }
}

/// GET /api/routing/rules - 列出路由规则
pub async fn list_rules(
    State(state): State<AppState>,
    Query(query): Query<serde_json::Value>,
) -> impl IntoResponse {
    let rules = if let Some(service_id) = query.get("service_id").and_then(|v| v.as_str()) {
        state.route_manager.get_rules_by_service(service_id)
    } else {
        state.route_manager.list_rules()
    };

    (StatusCode::OK, Json(ApiResponse::success(rules)))
}

/// DELETE /api/routing/rules/:rule_id - 删除路由规则
pub async fn delete_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
) -> impl IntoResponse {
    match state.route_manager.delete_rule(&rule_id) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success(()))),
        Err(e) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error(e))),
    }
}

/// PATCH /api/routing/rules/:rule_id - 更新路由规则
pub async fn update_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
    Json(req): Json<UpdateRuleRequest>,
) -> impl IntoResponse {
    match state.route_manager.get_rule(&rule_id) {
        Some(mut rule) => {
            if let Some(name) = req.name {
                rule.name = name;
            }
            if let Some(description) = req.description {
                rule.description = Some(description);
            }
            if let Some(strategy) = req.strategy {
                rule.strategy = strategy;
            }

            match state.route_manager.update_rule(rule.clone()) {
                Ok(_) => (StatusCode::OK, Json(ApiResponse::success(rule))),
                Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::<RouteRule>::error(e))),
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<RouteRule>::error(format!("Rule {} not found", rule_id))),
        ),
    }
}

/// POST /api/routing/rules/:rule_id/publish - 发布规则
pub async fn publish_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
) -> impl IntoResponse {
    match state.route_manager.publish_rule(&rule_id) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success(()))),
        Err(e) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error(e))),
    }
}

/// POST /api/routing/rules/:rule_id/unpublish - 停用规则
pub async fn unpublish_rule(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
) -> impl IntoResponse {
    match state.route_manager.unpublish_rule(&rule_id) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success(()))),
        Err(e) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error(e))),
    }
}

// ===== 路由规则分组关联 API =====

/// POST /api/routing/rules/:rule_id/groups - 添加分组到规则
pub async fn add_rule_group(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
    Json(req): Json<AddRuleGroupRequest>,
) -> impl IntoResponse {
    let group = RouteRuleGroup::with_location(
        rule_id.clone(),
        req.group_id,
        req.weight,
        req.region_id,
        req.zone_id,
    );

    match state.route_manager.add_rule_group(&rule_id, group) {
        Ok(_) => (StatusCode::CREATED, Json(ApiResponse::success(()))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::<()>::error(e))),
    }
}

/// GET /api/routing/rules/:rule_id/groups - 获取规则的分组
pub async fn get_rule_groups(
    State(state): State<AppState>,
    Path(rule_id): Path<String>,
) -> impl IntoResponse {
    let groups = state.route_manager.get_rule_groups(&rule_id);
    (StatusCode::OK, Json(ApiResponse::success(groups)))
}

/// DELETE /api/routing/rules/:rule_id/groups/:group_id - 移除分组
pub async fn remove_rule_group(
    State(state): State<AppState>,
    Path((rule_id, group_id)): Path<(String, String)>,
) -> impl IntoResponse {
    match state.route_manager.remove_rule_group(&rule_id, &group_id) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success(()))),
        Err(e) => (StatusCode::NOT_FOUND, Json(ApiResponse::<()>::error(e))),
    }
}

/// PATCH /api/routing/rules/:rule_id/groups/:group_id - 更新分组权重
pub async fn update_rule_group(
    State(state): State<AppState>,
    Path((rule_id, group_id)): Path<(String, String)>,
    Json(req): Json<UpdateRuleGroupRequest>,
) -> impl IntoResponse {
    // 获取现有分组配置
    let groups = state.route_manager.get_rule_groups(&rule_id);
    let group = groups.iter().find(|g| g.group_id == group_id);

    match group {
        Some(existing) => {
            let mut updated = existing.clone();
            updated.weight = req.weight;

            match state.route_manager.update_rule_group(&rule_id, updated.clone()) {
                Ok(_) => (StatusCode::OK, Json(ApiResponse::success(updated))),
                Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::<RouteRuleGroup>::error(e))),
            }
        }
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<RouteRuleGroup>::error(
                format!("Group {} not found in rule {}", group_id, rule_id)
            )),
        ),
    }
}

// ===== 分组实例绑定 API (Phase 19 新增) =====

/// 添加实例到分组请求
#[derive(Debug, Serialize, Deserialize)]
pub struct AddInstanceToGroupRequest {
    pub instance_id: String,
    pub region_id: String,
    pub zone_id: String,
    pub service_id: String,
    pub operator_id: String,
}

/// 批量添加服务实例请求
#[derive(Debug, Serialize, Deserialize)]
pub struct BatchAddServiceInstancesRequest {
    pub instances: Vec<GroupInstance>,
}

/// POST /api/routing/groups/:group_key/instances - 添加实例到分组 (手动绑定)
pub async fn add_instance_to_group(
    State(state): State<AppState>,
    Path(group_key): Path<String>,
    Json(req): Json<AddInstanceToGroupRequest>,
) -> impl IntoResponse {
    // 获取分组
    let group = match state.group_manager.get_group(&group_key) {
        Some(g) => g,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error(format!("Group {} not found", group_key))),
            );
        }
    };

    let group_id = match group.group_id {
        Some(id) => id,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Group has no ID".to_string())),
            );
        }
    };

    // 添加实例到分组
    match state
        .group_manager
        .add_instance_to_group(
            group_id,
            &req.instance_id,
            &req.region_id,
            &req.zone_id,
            &req.service_id,
            &req.operator_id,
        )
        .await
    {
        Ok(_) => (
            StatusCode::CREATED,
            Json(ApiResponse::success(())),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(e)),
        ),
    }
}

/// DELETE /api/routing/groups/:group_key/instances/:instance_id - 从分组移除实例
pub async fn remove_instance_from_group(
    State(state): State<AppState>,
    Path((group_key, instance_id)): Path<(String, String)>,
    Query(query): Query<GetGroupInstancesQuery>,
) -> impl IntoResponse {
    // 获取分组
    let group = match state.group_manager.get_group(&group_key) {
        Some(g) => g,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::<()>::error(format!("Group {} not found", group_key))),
            );
        }
    };

    let group_id = match group.group_id {
        Some(id) => id,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("Group has no ID".to_string())),
            );
        }
    };

    // 从请求中获取 region_id 和 zone_id (必需参数)
    let region_id = match query.region_id {
        Some(ref r) => r,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error("region_id is required".to_string())),
            );
        }
    };

    let zone_id = match query.zone_id {
        Some(ref z) => z,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error("zone_id is required".to_string())),
            );
        }
    };

    // 移除实例
    match state
        .group_manager
        .remove_instance_from_group(group_id, &instance_id, region_id, zone_id)
        .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success(())),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error(e)),
        ),
    }
}

/// POST /api/routing/services/:service_id/instances - 批量添加服务实例到分组
pub async fn batch_add_service_instances(
    State(state): State<AppState>,
    Path(service_id): Path<String>,
    Json(req): Json<BatchAddServiceInstancesRequest>,
) -> impl IntoResponse {
    // 验证所有实例属于同一服务
    for instance in &req.instances {
        if instance.service_id != service_id {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<usize>::error(format!(
                    "Instance service_id {} does not match path service_id {}",
                    instance.service_id, service_id
                ))),
            );
        }
    }

    // 获取第一个实例的 group_id (假设所有实例添加到同一分组)
    let group_id = match req.instances.first() {
        Some(inst) => inst.group_id,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<usize>::error("No instances provided".to_string())),
            );
        }
    };

    // 批量添加
    match state
        .group_manager
        .batch_add_service_instances(group_id, req.instances)
        .await
    {
        Ok(count) => (
            StatusCode::CREATED,
            Json(ApiResponse::success(count)),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<usize>::error(e)),
        ),
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use artemis_core::model::{GroupTag, GroupType, GroupStatus, RouteStrategy};

    // ===== ApiResponse 测试 =====

    #[test]
    fn test_api_response_success() {
        let response: ApiResponse<String> = ApiResponse::success("test".to_string());
        assert!(response.success);
        assert_eq!(response.data, Some("test".to_string()));
        assert!(response.message.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<String> = ApiResponse::error("error message".to_string());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.message, Some("error message".to_string()));
    }

    // ===== CreateGroupRequest 测试 =====

    #[test]
    fn test_create_group_request() {
        let req = CreateGroupRequest {
            service_id: "service1".to_string(),
            region_id: "us-east".to_string(),
            zone_id: "zone1".to_string(),
            name: "group1".to_string(),
            group_type: GroupType::Physical,
            description: Some("test group".to_string()),
        };
        assert_eq!(req.service_id, "service1");
        assert_eq!(req.name, "group1");
        assert_eq!(req.description, Some("test group".to_string()));
    }

    #[test]
    fn test_create_group_request_no_description() {
        let req = CreateGroupRequest {
            service_id: "service2".to_string(),
            region_id: "us-west".to_string(),
            zone_id: "zone2".to_string(),
            name: "group2".to_string(),
            group_type: GroupType::Logical,
            description: None,
        };
        assert_eq!(req.service_id, "service2");
        assert!(req.description.is_none());
    }

    // ===== CreateRuleRequest 测试 =====

    #[test]
    fn test_create_rule_request() {
        let req = CreateRuleRequest {
            route_id: "route1".to_string(),
            service_id: "service1".to_string(),
            name: "rule1".to_string(),
            description: Some("test rule".to_string()),
            strategy: RouteStrategy::WeightedRoundRobin,
        };
        assert_eq!(req.route_id, "route1");
        assert_eq!(req.name, "rule1");
    }

    // ===== AddRuleGroupRequest 测试 =====

    #[test]
    fn test_add_rule_group_request() {
        let req = AddRuleGroupRequest {
            group_id: "group1".to_string(),
            weight: 100,
            region_id: Some("us-east".to_string()),
            zone_id: Some("zone1".to_string()),
        };
        assert_eq!(req.group_id, "group1");
        assert_eq!(req.weight, 100);
        assert_eq!(req.region_id, Some("us-east".to_string()));
    }

    #[test]
    fn test_add_rule_group_request_no_location() {
        let req = AddRuleGroupRequest {
            group_id: "group2".to_string(),
            weight: 50,
            region_id: None,
            zone_id: None,
        };
        assert_eq!(req.weight, 50);
        assert!(req.region_id.is_none());
        assert!(req.zone_id.is_none());
    }

    // ===== ListGroupsQuery 测试 =====

    #[test]
    fn test_list_groups_query_by_service() {
        let query = ListGroupsQuery {
            service_id: Some("service1".to_string()),
            region_id: None,
        };
        assert_eq!(query.service_id, Some("service1".to_string()));
        assert!(query.region_id.is_none());
    }

    #[test]
    fn test_list_groups_query_by_region() {
        let query = ListGroupsQuery {
            service_id: None,
            region_id: Some("us-east".to_string()),
        };
        assert!(query.service_id.is_none());
        assert_eq!(query.region_id, Some("us-east".to_string()));
    }

    // ===== UpdateGroupRequest 测试 =====

    #[test]
    fn test_update_group_request_full() {
        let req = UpdateGroupRequest {
            description: Some("updated description".to_string()),
            status: Some(GroupStatus::Inactive),
        };
        assert_eq!(req.description, Some("updated description".to_string()));
        assert_eq!(req.status, Some(GroupStatus::Inactive));
    }

    #[test]
    fn test_update_group_request_partial() {
        let req = UpdateGroupRequest {
            description: Some("updated".to_string()),
            status: None,
        };
        assert!(req.description.is_some());
        assert!(req.status.is_none());
    }

    // ===== UpdateRuleRequest 测试 =====

    #[test]
    fn test_update_rule_request() {
        let req = UpdateRuleRequest {
            name: Some("new name".to_string()),
            description: Some("new desc".to_string()),
            strategy: Some(RouteStrategy::CloseByVisit),
        };
        assert_eq!(req.name, Some("new name".to_string()));
        assert!(req.strategy.is_some());
    }

    // ===== UpdateRuleGroupRequest 测试 =====

    #[test]
    fn test_update_rule_group_request() {
        let req = UpdateRuleGroupRequest {
            weight: 75,
        };
        assert_eq!(req.weight, 75);
    }

    // ===== AddGroupTagsRequest 测试 =====

    #[test]
    fn test_add_group_tags_request() {
        let tags = vec![
            GroupTag {
                key: "env".to_string(),
                value: "prod".to_string(),
            },
            GroupTag {
                key: "team".to_string(),
                value: "platform".to_string(),
            },
        ];
        let req = AddGroupTagsRequest {
            tags: tags.clone(),
        };
        assert_eq!(req.tags.len(), 2);
        assert_eq!(req.tags[0].key, "env");
        assert_eq!(req.tags[1].value, "platform");
    }

    // ===== GetGroupInstancesQuery 测试 =====

    #[test]
    fn test_get_group_instances_query() {
        let query = GetGroupInstancesQuery {
            region_id: Some("us-east".to_string()),
            zone_id: Some("zone1".to_string()),
        };
        assert_eq!(query.region_id, Some("us-east".to_string()));
        assert_eq!(query.zone_id, Some("zone1".to_string()));
    }

    // ===== AddInstanceToGroupRequest 测试 =====

    #[test]
    fn test_add_instance_to_group_request() {
        let req = AddInstanceToGroupRequest {
            instance_id: "inst1".to_string(),
            region_id: "us-east".to_string(),
            zone_id: "zone1".to_string(),
            service_id: "service1".to_string(),
            operator_id: "admin".to_string(),
        };
        assert_eq!(req.instance_id, "inst1");
        assert_eq!(req.operator_id, "admin");
    }

    // ===== BatchAddServiceInstancesRequest 测试 =====

    #[test]
    fn test_batch_add_service_instances_request() {
        use artemis_core::model::group::BindingType;
        let instances = vec![
            GroupInstance {
                id: None,
                group_id: 1,
                instance_id: "inst1".to_string(),
                region_id: "us-east".to_string(),
                zone_id: "zone1".to_string(),
                service_id: "service1".to_string(),
                binding_type: Some(BindingType::Manual),
                operator_id: Some("admin".to_string()),
                created_at: None,
            },
        ];
        let req = BatchAddServiceInstancesRequest {
            instances: instances.clone(),
        };
        assert_eq!(req.instances.len(), 1);
        assert_eq!(req.instances[0].instance_id, "inst1");
    }

    #[test]
    fn test_batch_add_service_instances_request_empty() {
        let req = BatchAddServiceInstancesRequest {
            instances: vec![],
        };
        assert!(req.instances.is_empty());
    }

    #[test]
    fn test_batch_add_service_instances_request_multiple() {
        use artemis_core::model::group::BindingType;
        let instances = vec![
            GroupInstance {
                id: None,
                group_id: 1,
                instance_id: "inst1".to_string(),
                region_id: "us-east".to_string(),
                zone_id: "zone1".to_string(),
                service_id: "service1".to_string(),
                binding_type: Some(BindingType::Manual),
                operator_id: Some("admin".to_string()),
                created_at: None,
            },
            GroupInstance {
                id: None,
                group_id: 1,
                instance_id: "inst2".to_string(),
                region_id: "us-east".to_string(),
                zone_id: "zone1".to_string(),
                service_id: "service1".to_string(),
                binding_type: Some(BindingType::Auto),
                operator_id: None,
                created_at: None,
            },
        ];
        let req = BatchAddServiceInstancesRequest {
            instances: instances.clone(),
        };
        assert_eq!(req.instances.len(), 2);
        assert_eq!(req.instances[0].instance_id, "inst1");
        assert_eq!(req.instances[1].instance_id, "inst2");
    }

    // ===== UpdateGroupRequest 边界测试 =====

    #[test]
    fn test_update_group_request_empty() {
        let req = UpdateGroupRequest {
            description: None,
            status: None,
        };
        assert!(req.description.is_none());
        assert!(req.status.is_none());
    }

    #[test]
    fn test_update_group_request_description_only() {
        let req = UpdateGroupRequest {
            description: Some("new description".to_string()),
            status: None,
        };
        assert_eq!(req.description, Some("new description".to_string()));
        assert!(req.status.is_none());
    }

    #[test]
    fn test_update_group_request_status_only() {
        let req = UpdateGroupRequest {
            description: None,
            status: Some(GroupStatus::Active),
        };
        assert!(req.description.is_none());
        assert_eq!(req.status, Some(GroupStatus::Active));
    }

    // ===== UpdateRuleRequest 边界测试 =====

    #[test]
    fn test_update_rule_request_empty() {
        let req = UpdateRuleRequest {
            name: None,
            description: None,
            strategy: None,
        };
        assert!(req.name.is_none());
        assert!(req.description.is_none());
        assert!(req.strategy.is_none());
    }

    #[test]
    fn test_update_rule_request_name_only() {
        let req = UpdateRuleRequest {
            name: Some("updated rule".to_string()),
            description: None,
            strategy: None,
        };
        assert_eq!(req.name, Some("updated rule".to_string()));
        assert!(req.description.is_none());
    }

    #[test]
    fn test_update_rule_request_description_only() {
        let req = UpdateRuleRequest {
            name: None,
            description: Some("updated desc".to_string()),
            strategy: None,
        };
        assert!(req.name.is_none());
        assert_eq!(req.description, Some("updated desc".to_string()));
    }

    #[test]
    fn test_update_rule_request_strategy_only() {
        let req = UpdateRuleRequest {
            name: None,
            description: None,
            strategy: Some(RouteStrategy::WeightedRoundRobin),
        };
        assert!(req.name.is_none());
        assert!(req.strategy.is_some());
    }

    // ===== ListGroupsQuery 边界测试 =====

    #[test]
    fn test_list_groups_query_empty() {
        let query = ListGroupsQuery {
            service_id: None,
            region_id: None,
        };
        assert!(query.service_id.is_none());
        assert!(query.region_id.is_none());
    }

    // ===== GetGroupInstancesQuery 边界测试 =====

    #[test]
    fn test_get_group_instances_query_empty() {
        let query = GetGroupInstancesQuery {
            region_id: None,
            zone_id: None,
        };
        assert!(query.region_id.is_none());
        assert!(query.zone_id.is_none());
    }

    #[test]
    fn test_get_group_instances_query_region_only() {
        let query = GetGroupInstancesQuery {
            region_id: Some("us-west".to_string()),
            zone_id: None,
        };
        assert_eq!(query.region_id, Some("us-west".to_string()));
        assert!(query.zone_id.is_none());
    }

    #[test]
    fn test_get_group_instances_query_zone_only() {
        let query = GetGroupInstancesQuery {
            region_id: None,
            zone_id: Some("zone2".to_string()),
        };
        assert!(query.region_id.is_none());
        assert_eq!(query.zone_id, Some("zone2".to_string()));
    }

    // ===== AddInstanceToGroupRequest 边界测试 =====

    #[test]
    fn test_add_instance_to_group_request_different_regions() {
        let req1 = AddInstanceToGroupRequest {
            instance_id: "inst-us-east".to_string(),
            region_id: "us-east".to_string(),
            zone_id: "zone1".to_string(),
            service_id: "service1".to_string(),
            operator_id: "admin".to_string(),
        };
        let req2 = AddInstanceToGroupRequest {
            instance_id: "inst-eu-west".to_string(),
            region_id: "eu-west".to_string(),
            zone_id: "zone2".to_string(),
            service_id: "service1".to_string(),
            operator_id: "system".to_string(),
        };
        assert_eq!(req1.region_id, "us-east");
        assert_eq!(req2.region_id, "eu-west");
    }

    // ===== AddGroupTagsRequest 边界测试 =====

    #[test]
    fn test_add_group_tags_request_empty() {
        let req = AddGroupTagsRequest {
            tags: vec![],
        };
        assert!(req.tags.is_empty());
    }

    #[test]
    fn test_add_group_tags_request_single() {
        let req = AddGroupTagsRequest {
            tags: vec![
                GroupTag {
                    key: "version".to_string(),
                    value: "1.0.0".to_string(),
                },
            ],
        };
        assert_eq!(req.tags.len(), 1);
        assert_eq!(req.tags[0].key, "version");
        assert_eq!(req.tags[0].value, "1.0.0");
    }
}
