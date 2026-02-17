use crate::state::AppState;
use artemis_management::auth::{UserRole, UserStatus, UserResponse, Session};
use axum::{
    Extension,
    extract::{Path, Request, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};

// ===== 请求/响应模型 =====

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResetPasswordRequest {
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: Option<String>,
    pub description: Option<String>,
    pub password: String,
    pub role: String, // "admin", "operator", "viewer"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub description: Option<String>,
    pub role: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserStatusRequest {
    pub status: String, // "active", "inactive"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckPermissionRequest {
    pub resource: String,
    pub action: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckPermissionResponse {
    pub allowed: bool,
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

// ===== 辅助函数 =====

/// 从请求中提取当前用户ID
fn extract_user_id(req: &Request) -> Result<String, (StatusCode, String)> {
    req.extensions()
        .get::<String>()
        .cloned()
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "User ID not found in request".to_string()))
}

// ===== 认证 API =====

/// POST /api/auth/login - 用户登录
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> impl IntoResponse {
    match state.auth_manager.authenticate(&req.username, &req.password, None, None) {
        Ok(token) => {
            let response = ApiResponse::success(LoginResponse {
                access_token: token,
                token_type: "Bearer".to_string(),
                expires_in: 3600,
            });
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            let response = ApiResponse::<LoginResponse>::error(e);
            (StatusCode::UNAUTHORIZED, Json(response))
        }
    }
}

/// POST /api/auth/logout - 用户登出
pub async fn logout(
    State(state): State<AppState>,
    req: Request,
) -> impl IntoResponse {
    // 从 header 中提取 token
    let token = req
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "));

    if let Some(token) = token {
        match state.auth_manager.logout(token) {
            Ok(_) => (StatusCode::OK, Json(ApiResponse::success("Logged out successfully".to_string()))),
            Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::<String>::error(e))),
        }
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<String>::error("Missing token".to_string())),
        )
    }
}

/// POST /api/auth/refresh - 刷新 token
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshTokenRequest>,
) -> impl IntoResponse {
    match state.auth_manager.refresh_token(&req.token) {
        Ok(new_token) => {
            let response = ApiResponse::success(LoginResponse {
                access_token: new_token,
                token_type: "Bearer".to_string(),
                expires_in: 3600,
            });
            (StatusCode::OK, Json(response))
        }
        Err(e) => {
            let response = ApiResponse::<LoginResponse>::error(e);
            (StatusCode::UNAUTHORIZED, Json(response))
        }
    }
}

/// GET /api/auth/user - 获取当前用户信息
pub async fn get_current_user(
    State(state): State<AppState>,
    req: Request,
) -> impl IntoResponse {
    match extract_user_id(&req) {
        Ok(user_id) => {
            match state.auth_manager.get_user(&user_id) {
                Some(user) => (StatusCode::OK, Json(ApiResponse::success(user.to_response()))),
                None => (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::<UserResponse>::error("User not found".to_string())),
                ),
            }
        }
        Err((status, msg)) => (status, Json(ApiResponse::<UserResponse>::error(msg))),
    }
}

/// GET /api/auth/permissions - 获取用户权限
pub async fn get_user_permissions(
    State(state): State<AppState>,
    req: Request,
) -> impl IntoResponse {
    match extract_user_id(&req) {
        Ok(user_id) => {
            let permissions = state.auth_manager.get_user_permissions(&user_id);
            (StatusCode::OK, Json(ApiResponse::success(permissions)))
        }
        Err((status, msg)) => (status, Json(ApiResponse::<Vec<String>>::error(msg))),
    }
}

/// POST /api/auth/password/change - 修改密码
pub async fn change_password(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Json(body): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    match state.auth_manager.change_password(&user_id, &body.old_password, &body.new_password) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success("Password changed successfully".to_string()))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::<String>::error(e))),
    }
}

/// POST /api/auth/password/reset/:user_id - 重置密码（管理员）
pub async fn reset_password(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(req): Json<ResetPasswordRequest>,
) -> impl IntoResponse {
    match state.auth_manager.reset_password(&user_id, &req.new_password) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success("Password reset successfully".to_string()))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::<String>::error(e))),
    }
}

/// GET /api/auth/sessions - 列出用户会话
pub async fn list_sessions(
    State(state): State<AppState>,
    req: Request,
) -> impl IntoResponse {
    match extract_user_id(&req) {
        Ok(user_id) => {
            let sessions = state.auth_manager.list_user_sessions(&user_id);
            (StatusCode::OK, Json(ApiResponse::success(sessions)))
        }
        Err((status, msg)) => (status, Json(ApiResponse::<Vec<Session>>::error(msg))),
    }
}

/// DELETE /api/auth/sessions/:session_id - 撤销会话
pub async fn revoke_session(
    State(state): State<AppState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    match state.auth_manager.revoke_session(&session_id) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success("Session revoked successfully".to_string()))),
        Err(e) => (StatusCode::NOT_FOUND, Json(ApiResponse::<String>::error(e))),
    }
}

/// GET /api/auth/roles - 列出所有角色
pub async fn list_roles() -> impl IntoResponse {
    let roles = vec!["admin", "operator", "viewer"];
    (StatusCode::OK, Json(ApiResponse::success(roles)))
}

/// POST /api/auth/check-permission - 检查权限
pub async fn check_permission(
    State(state): State<AppState>,
    Extension(user_id): Extension<String>,
    Json(body): Json<CheckPermissionRequest>,
) -> impl IntoResponse {
    let allowed = state.auth_manager.check_permission(&user_id, &body.resource, &body.action);
    (
        StatusCode::OK,
        Json(ApiResponse::success(CheckPermissionResponse { allowed })),
    )
}

// ===== 用户管理 API =====

/// GET /api/auth/users - 列出所有用户
pub async fn list_users(State(state): State<AppState>) -> impl IntoResponse {
    let users: Vec<UserResponse> = state
        .auth_manager
        .list_users()
        .into_iter()
        .map(|u| u.to_response())
        .collect();
    (StatusCode::OK, Json(ApiResponse::success(users)))
}

/// POST /api/auth/users - 创建用户
pub async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> impl IntoResponse {
    let role = match UserRole::from_str(&req.role) {
        Some(r) => r,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<UserResponse>::error(format!("Invalid role: {}", req.role))),
            );
        }
    };

    match state.auth_manager.create_user(&req.username, req.email, req.description, &req.password, role) {
        Ok(user) => (StatusCode::CREATED, Json(ApiResponse::success(user.to_response()))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::<UserResponse>::error(e))),
    }
}

/// GET /api/auth/users/:user_id - 获取用户详情
pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    match state.auth_manager.get_user(&user_id) {
        Some(user) => (StatusCode::OK, Json(ApiResponse::success(user.to_response()))),
        None => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<UserResponse>::error("User not found".to_string())),
        ),
    }
}

/// PUT /api/auth/users/:user_id - 更新用户
pub async fn update_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(req): Json<UpdateUserRequest>,
) -> impl IntoResponse {
    let role = if let Some(role_str) = req.role {
        match UserRole::from_str(&role_str) {
            Some(r) => Some(r),
            None => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::<UserResponse>::error(format!("Invalid role: {}", role_str))),
                );
            }
        }
    } else {
        None
    };

    match state.auth_manager.update_user(&user_id, req.email, req.description, role) {
        Ok(user) => (StatusCode::OK, Json(ApiResponse::success(user.to_response()))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::<UserResponse>::error(e))),
    }
}

/// DELETE /api/auth/users/:user_id - 删除用户
pub async fn delete_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    match state.auth_manager.delete_user(&user_id) {
        Ok(_) => (StatusCode::OK, Json(ApiResponse::success("User deleted successfully".to_string()))),
        Err(e) => (StatusCode::NOT_FOUND, Json(ApiResponse::<String>::error(e))),
    }
}

/// PATCH /api/auth/users/:user_id/status - 修改用户状态
pub async fn update_user_status(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(req): Json<UpdateUserStatusRequest>,
) -> impl IntoResponse {
    let status = match UserStatus::from_str(&req.status) {
        Some(s) => s,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<UserResponse>::error(format!("Invalid status: {}", req.status))),
            );
        }
    };

    match state.auth_manager.change_user_status(&user_id, status) {
        Ok(user) => (StatusCode::OK, Json(ApiResponse::success(user.to_response()))),
        Err(e) => (StatusCode::BAD_REQUEST, Json(ApiResponse::<UserResponse>::error(e))),
    }
}

/// GET /api/auth/users/:user_id/login-history - 获取登录历史
pub async fn get_login_history(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> impl IntoResponse {
    let history = state.auth_manager.get_login_history(&user_id, 50);
    (StatusCode::OK, Json(ApiResponse::success(history)))
}
