use crate::state::AppState;
use artemis_core::{model::*, traits::DiscoveryService};
use axum::{Json, extract::State};

pub async fn get_service(
    State(state): State<AppState>,
    Json(request): Json<GetServiceRequest>,
) -> Json<GetServiceResponse> {
    Json(state.discovery_service.get_service(request).await)
}

pub async fn get_services(State(state): State<AppState>) -> Json<GetServicesResponse> {
    let request =
        GetServicesRequest { region_id: "default".to_string(), zone_id: "default".to_string() };
    Json(state.discovery_service.get_services(request).await)
}
