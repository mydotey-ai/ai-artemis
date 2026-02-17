//! Management API endpoints for instance pull-in/pull-out operations

use crate::state::AppState;
use artemis_core::model::ResponseStatus;
use artemis_management::model::{
    GetAllInstanceOperationsRequest, GetAllInstanceOperationsResponse,
    GetAllServerOperationsRequest, GetAllServerOperationsResponse, GetInstanceOperationsRequest,
    GetInstanceOperationsResponse, InstanceOperation, IsInstanceDownRequest,
    IsInstanceDownResponse, IsServerDownRequest, IsServerDownResponse, OperateInstanceRequest,
    OperateInstanceResponse, OperateServerRequest, OperateServerResponse, ServerOperationInfo,
};
use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use tracing::{error, info};

// ========== Instance Operations ==========

/// POST /api/management/instance/operate-instance.json
/// 操作实例 (拉入/拉出)
pub async fn operate_instance(
    State(state): State<AppState>,
    Json(req): Json<OperateInstanceRequest>,
) -> Response {
    info!(
        "Operate instance: {:?}, operation: {:?}, complete: {}",
        req.instance_key, req.operation, req.operation_complete
    );

    let result = match req.operation {
        InstanceOperation::PullOut => state.instance_manager.pull_out_instance(
            &req.instance_key,
            req.operator_id.clone(),
            req.operation_complete,
        ),
        InstanceOperation::PullIn => state.instance_manager.pull_in_instance(
            &req.instance_key,
            req.operator_id.clone(),
            req.operation_complete,
        ),
    };

    match result {
        Ok(_) => {
            let response = OperateInstanceResponse { status: ResponseStatus::success() };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to operate instance: {}", e);
            let response = OperateInstanceResponse {
                status: ResponseStatus::error(
                    artemis_core::model::ErrorCode::InternalError,
                    format!("Operation failed: {}", e),
                ),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// POST /api/management/instance/get-instance-operations.json
/// 查询实例操作列表
pub async fn get_instance_operations(
    State(state): State<AppState>,
    Json(req): Json<GetInstanceOperationsRequest>,
) -> Response {
    info!("Get instance operations: {:?}", req.instance_key);

    let operations = state.instance_manager.get_instance_operations(&req.instance_key);

    let response = GetInstanceOperationsResponse { status: ResponseStatus::success(), operations };

    (StatusCode::OK, Json(response)).into_response()
}

/// POST /api/management/instance/is-instance-down.json
/// 查询实例是否被拉出
pub async fn is_instance_down(
    State(state): State<AppState>,
    Json(req): Json<IsInstanceDownRequest>,
) -> Response {
    let is_down = state.instance_manager.is_instance_down(&req.instance_key);

    info!("Is instance down: {:?}, result: {}", req.instance_key, is_down);

    let response = IsInstanceDownResponse { status: ResponseStatus::success(), is_down };

    (StatusCode::OK, Json(response)).into_response()
}

// ========== Server Operations ==========

/// POST /api/management/server/operate-server.json
/// 操作服务器 (批量拉入/拉出)
pub async fn operate_server(
    State(state): State<AppState>,
    Json(req): Json<OperateServerRequest>,
) -> Response {
    info!(
        "Operate server: {}, region: {}, operation: {:?}, complete: {}",
        req.server_id, req.region_id, req.operation, req.operation_complete
    );

    let result = match req.operation {
        artemis_management::model::ServerOperation::PullOut => {
            state.instance_manager.pull_out_server(
                &req.server_id,
                &req.region_id,
                req.operator_id.clone(),
                req.operation_complete,
            )
        }
        artemis_management::model::ServerOperation::PullIn => {
            state.instance_manager.pull_in_server(
                &req.server_id,
                &req.region_id,
                req.operator_id.clone(),
                req.operation_complete,
            )
        }
    };

    match result {
        Ok(_) => {
            let response = OperateServerResponse { status: ResponseStatus::success() };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to operate server: {}", e);
            let response = OperateServerResponse {
                status: ResponseStatus::error(
                    artemis_core::model::ErrorCode::InternalError,
                    format!("Operation failed: {}", e),
                ),
            };
            (StatusCode::INTERNAL_SERVER_ERROR, Json(response)).into_response()
        }
    }
}

/// POST /api/management/server/is-server-down.json
/// 查询服务器是否被拉出
pub async fn is_server_down(
    State(state): State<AppState>,
    Json(req): Json<IsServerDownRequest>,
) -> Response {
    let is_down = state.instance_manager.is_server_down(&req.server_id, &req.region_id);

    info!("Is server down: {}, region: {}, result: {}", req.server_id, req.region_id, is_down);

    let response = IsServerDownResponse { status: ResponseStatus::success(), is_down };

    (StatusCode::OK, Json(response)).into_response()
}

// ========== Phase 25: 批量操作查询 API ==========

/// POST /api/management/all-instance-operations.json
/// 查询所有实例操作 (POST 版本)
pub async fn get_all_instance_operations_post(
    State(state): State<AppState>,
    Json(req): Json<GetAllInstanceOperationsRequest>,
) -> Response {
    info!("Get all instance operations (POST), region_id: {:?}", req.region_id);

    let records = state.instance_manager.get_all_instance_operations(req.region_id.as_deref());

    let response = GetAllInstanceOperationsResponse {
        status: ResponseStatus::success(),
        instance_operation_records: records,
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// GET /api/management/all-instance-operations.json?regionId=X
/// 查询所有实例操作 (GET 版本,支持 query parameter)
#[derive(Debug, Deserialize)]
pub struct AllInstanceOperationsQuery {
    #[serde(rename = "regionId")]
    pub region_id: Option<String>,
}

pub async fn get_all_instance_operations_get(
    State(state): State<AppState>,
    Query(query): Query<AllInstanceOperationsQuery>,
) -> Response {
    info!("Get all instance operations (GET), region_id: {:?}", query.region_id);

    let records = state.instance_manager.get_all_instance_operations(query.region_id.as_deref());

    let response = GetAllInstanceOperationsResponse {
        status: ResponseStatus::success(),
        instance_operation_records: records,
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// POST /api/management/all-server-operations.json
/// 查询所有服务器操作 (POST 版本)
pub async fn get_all_server_operations_post(
    State(state): State<AppState>,
    Json(req): Json<GetAllServerOperationsRequest>,
) -> Response {
    info!("Get all server operations (POST), region_id: {:?}", req.region_id);

    let records = state.instance_manager.get_all_server_operations(req.region_id.as_deref());

    // 转换为 ServerOperationInfo
    let server_records: Vec<ServerOperationInfo> = records
        .into_iter()
        .map(|(server_id, region_id, operation)| ServerOperationInfo {
            server_id,
            region_id,
            operation,
        })
        .collect();

    let response = GetAllServerOperationsResponse {
        status: ResponseStatus::success(),
        server_operation_records: server_records,
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// GET /api/management/all-server-operations.json?regionId=X
/// 查询所有服务器操作 (GET 版本,支持 query parameter)
#[derive(Debug, Deserialize)]
pub struct AllServerOperationsQuery {
    #[serde(rename = "regionId")]
    pub region_id: Option<String>,
}

pub async fn get_all_server_operations_get(
    State(state): State<AppState>,
    Query(query): Query<AllServerOperationsQuery>,
) -> Response {
    info!("Get all server operations (GET), region_id: {:?}", query.region_id);

    let records = state.instance_manager.get_all_server_operations(query.region_id.as_deref());

    // 转换为 ServerOperationInfo
    let server_records: Vec<ServerOperationInfo> = records
        .into_iter()
        .map(|(server_id, region_id, operation)| ServerOperationInfo {
            server_id,
            region_id,
            operation,
        })
        .collect();

    let response = GetAllServerOperationsResponse {
        status: ResponseStatus::success(),
        server_operation_records: server_records,
    };

    (StatusCode::OK, Json(response)).into_response()
}
