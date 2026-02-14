//! 路由管理 HTTP API
//!
//! 提供分组和路由规则管理的 REST API

use crate::state::AppState;
use artemis_core::model::{
    GroupStatus, GroupType, RouteRule, RouteRuleGroup, RouteRuleStatus, RouteStrategy,
    ServiceGroup,
};
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
