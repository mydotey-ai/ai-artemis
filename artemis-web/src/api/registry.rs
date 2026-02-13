use crate::state::AppState;
use artemis_core::{model::*, traits::RegistryService};
use axum::{Json, extract::State};

pub async fn register(
    State(state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> Json<RegisterResponse> {
    Json(state.registry_service.register(request).await)
}

pub async fn heartbeat(
    State(state): State<AppState>,
    Json(request): Json<HeartbeatRequest>,
) -> Json<HeartbeatResponse> {
    Json(state.registry_service.heartbeat(request).await)
}

pub async fn unregister(
    State(state): State<AppState>,
    Json(request): Json<UnregisterRequest>,
) -> Json<UnregisterResponse> {
    Json(state.registry_service.unregister(request).await)
}
