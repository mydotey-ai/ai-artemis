use crate::web::state::ManagementState;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

/// JWT 认证中间件
///
/// 从 Authorization header 中提取 Bearer token,验证并将 user_id 注入到请求扩展中
pub async fn jwt_auth(
    State(state): State<ManagementState>,
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, &'static str)> {
    // 从 header 中提取 token
    let auth_header = headers.get("Authorization").and_then(|h| h.to_str().ok());

    let token = auth_header
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or((StatusCode::UNAUTHORIZED, "Missing or invalid Authorization header"))?;

    // 验证 token
    let session = state
        .auth_manager
        .validate_token(token)
        .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid or expired token"))?;

    // 将 user_id 注入到请求扩展中
    req.extensions_mut().insert(session.user_id.clone());

    Ok(next.run(req).await)
}

/// 从请求中提取当前用户ID
///
/// 仅在 jwt_auth 中间件之后可用
pub fn extract_user_id(req: &Request) -> Option<String> {
    req.extensions().get::<String>().cloned()
}
