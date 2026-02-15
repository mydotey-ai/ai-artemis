use artemis_core::model::{
    GetClusterNodeStatusRequest, GetClusterStatusRequest, GetConfigStatusRequest,
    GetDeploymentStatusRequest, GetLeasesStatusRequest,
};
use axum::{
    extract::{Json, Query, State},
    response::IntoResponse,
};
use serde::Deserialize;
use crate::state::AppState;

// ==================== Node Status ====================

pub async fn get_cluster_node_status_post(
    State(state): State<AppState>,
    Json(_request): Json<GetClusterNodeStatusRequest>,
) -> impl IntoResponse {
    let response = state.status_service
        .get_cluster_node_status(GetClusterNodeStatusRequest {})
        .await;
    Json(response)
}

pub async fn get_cluster_node_status_get(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let response = state.status_service
        .get_cluster_node_status(GetClusterNodeStatusRequest {})
        .await;
    Json(response)
}

// ==================== Cluster Status ====================

pub async fn get_cluster_status_post(
    State(state): State<AppState>,
    Json(_request): Json<GetClusterStatusRequest>,
) -> impl IntoResponse {
    let response = state.status_service
        .get_cluster_status(GetClusterStatusRequest {})
        .await;
    Json(response)
}

pub async fn get_cluster_status_get(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let response = state.status_service
        .get_cluster_status(GetClusterStatusRequest {})
        .await;
    Json(response)
}

// ==================== Leases Status ====================

#[derive(Debug, Deserialize)]
pub struct GetLeasesQuery {
    #[serde(rename = "appIds")]
    pub app_ids: Option<Vec<String>>,
}

pub async fn get_leases_status_post(
    State(state): State<AppState>,
    Json(request): Json<GetLeasesStatusRequest>,
) -> impl IntoResponse {
    let response = state.status_service.get_leases_status(request).await;
    Json(response)
}

pub async fn get_leases_status_get(
    State(state): State<AppState>,
    Query(query): Query<GetLeasesQuery>,
) -> impl IntoResponse {
    let request = GetLeasesStatusRequest {
        service_ids: query.app_ids,
    };
    let response = state.status_service.get_leases_status(request).await;
    Json(response)
}

pub async fn get_legacy_leases_status_post(
    State(state): State<AppState>,
    Json(request): Json<GetLeasesStatusRequest>,
) -> impl IntoResponse {
    let response = state.status_service.get_legacy_leases_status(request).await;
    Json(response)
}

pub async fn get_legacy_leases_status_get(
    State(state): State<AppState>,
    Query(query): Query<GetLeasesQuery>,
) -> impl IntoResponse {
    let request = GetLeasesStatusRequest {
        service_ids: query.app_ids,
    };
    let response = state.status_service.get_legacy_leases_status(request).await;
    Json(response)
}

// ==================== Config Status ====================

pub async fn get_config_status_post(
    State(state): State<AppState>,
    Json(_request): Json<GetConfigStatusRequest>,
) -> impl IntoResponse {
    let response = state.status_service
        .get_config_status(GetConfigStatusRequest {})
        .await;
    Json(response)
}

pub async fn get_config_status_get(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let response = state.status_service
        .get_config_status(GetConfigStatusRequest {})
        .await;
    Json(response)
}

// ==================== Deployment Status ====================

pub async fn get_deployment_status_post(
    State(state): State<AppState>,
    Json(_request): Json<GetDeploymentStatusRequest>,
) -> impl IntoResponse {
    let response = state.status_service
        .get_deployment_status(GetDeploymentStatusRequest {})
        .await;
    Json(response)
}

pub async fn get_deployment_status_get(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let response = state.status_service
        .get_deployment_status(GetDeploymentStatusRequest {})
        .await;
    Json(response)
}
