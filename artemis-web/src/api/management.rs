//! Management API endpoints for instance pull-in/pull-out operations

use crate::state::AppState;
use artemis_core::model::{
    GetInstanceOperationsRequest, GetInstanceOperationsResponse, InstanceOperation,
    IsInstanceDownRequest, IsInstanceDownResponse, IsServerDownRequest, IsServerDownResponse,
    OperateInstanceRequest, OperateInstanceResponse, OperateServerRequest,
    OperateServerResponse, ResponseStatus,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
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
            let response = OperateInstanceResponse {
                status: ResponseStatus::success(),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to operate instance: {}", e);
            let response = OperateInstanceResponse {
                status: ResponseStatus::error(artemis_core::model::ErrorCode::InternalError, format!("Operation failed: {}", e)),
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

    let operations = state
        .instance_manager
        .get_instance_operations(&req.instance_key);

    let response = GetInstanceOperationsResponse {
        status: ResponseStatus::success(),
        operations,
    };

    (StatusCode::OK, Json(response)).into_response()
}

/// POST /api/management/instance/is-instance-down.json
/// 查询实例是否被拉出
pub async fn is_instance_down(
    State(state): State<AppState>,
    Json(req): Json<IsInstanceDownRequest>,
) -> Response {
    let is_down = state.instance_manager.is_instance_down(&req.instance_key);

    info!(
        "Is instance down: {:?}, result: {}",
        req.instance_key, is_down
    );

    let response = IsInstanceDownResponse {
        status: ResponseStatus::success(),
        is_down,
    };

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
        artemis_core::model::ServerOperation::PullOut => state.instance_manager.pull_out_server(
            &req.server_id,
            &req.region_id,
            req.operator_id.clone(),
            req.operation_complete,
        ),
        artemis_core::model::ServerOperation::PullIn => state.instance_manager.pull_in_server(
            &req.server_id,
            &req.region_id,
            req.operator_id.clone(),
            req.operation_complete,
        ),
    };

    match result {
        Ok(_) => {
            let response = OperateServerResponse {
                status: ResponseStatus::success(),
            };
            (StatusCode::OK, Json(response)).into_response()
        }
        Err(e) => {
            error!("Failed to operate server: {}", e);
            let response = OperateServerResponse {
                status: ResponseStatus::error(artemis_core::model::ErrorCode::InternalError, format!("Operation failed: {}", e)),
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
    let is_down = state
        .instance_manager
        .is_server_down(&req.server_id, &req.region_id);

    info!(
        "Is server down: {}, region: {}, result: {}",
        req.server_id, req.region_id, is_down
    );

    let response = IsServerDownResponse {
        status: ResponseStatus::success(),
        is_down,
    };

    (StatusCode::OK, Json(response)).into_response()
}
