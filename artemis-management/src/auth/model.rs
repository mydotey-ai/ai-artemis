use serde::{Deserialize, Serialize};

/// 用户角色
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Operator,
    Viewer,
}

impl UserRole {
    pub fn as_str(&self) -> &str {
        match self {
            UserRole::Admin => "admin",
            UserRole::Operator => "operator",
            UserRole::Viewer => "viewer",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "admin" => Some(UserRole::Admin),
            "operator" => Some(UserRole::Operator),
            "viewer" => Some(UserRole::Viewer),
            _ => None,
        }
    }
}

/// 用户状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Inactive,
}

impl UserStatus {
    pub fn as_str(&self) -> &str {
        match self {
            UserStatus::Active => "active",
            UserStatus::Inactive => "inactive",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "active" => Some(UserStatus::Active),
            "inactive" => Some(UserStatus::Inactive),
            _ => None,
        }
    }
}

/// 用户模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub user_id: String,
    pub username: String,
    pub email: Option<String>,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub created_at: i64,
    pub updated_at: i64,
}

impl User {
    pub fn new(
        username: String,
        email: Option<String>,
        password_hash: String,
        role: UserRole,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            user_id: uuid::Uuid::new_v4().to_string(),
            username,
            email,
            password_hash,
            role,
            status: UserStatus::Active,
            created_at: now,
            updated_at: now,
        }
    }

    /// 用户响应(不包含密码)
    pub fn to_response(&self) -> UserResponse {
        UserResponse {
            user_id: self.user_id.clone(),
            username: self.username.clone(),
            email: self.email.clone(),
            role: self.role.clone(),
            status: self.status.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

/// 用户响应(不包含密码)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    pub user_id: String,
    pub username: String,
    pub email: Option<String>,
    pub role: UserRole,
    pub status: UserStatus,
    pub created_at: i64,
    pub updated_at: i64,
}

/// 会话模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub session_id: String,
    pub user_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: i64,
    pub expires_at: i64,
    pub last_activity: i64,
}

impl Session {
    pub fn new(
        user_id: String,
        token: String,
        ip_address: Option<String>,
        user_agent: Option<String>,
        expiry_seconds: i64,
    ) -> Self {
        let now = chrono::Utc::now().timestamp();
        Self {
            session_id: uuid::Uuid::new_v4().to_string(),
            user_id,
            token: Some(token),
            ip_address,
            user_agent,
            created_at: now,
            expires_at: now + expiry_seconds,
            last_activity: now,
        }
    }

    pub fn is_expired(&self) -> bool {
        chrono::Utc::now().timestamp() > self.expires_at
    }
}

/// 登录状态
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LoginStatus {
    Success,
    Failed,
}

impl LoginStatus {
    pub fn as_str(&self) -> &str {
        match self {
            LoginStatus::Success => "success",
            LoginStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "success" => Some(LoginStatus::Success),
            "failed" => Some(LoginStatus::Failed),
            _ => None,
        }
    }
}

/// 登录历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginHistory {
    pub id: i64,
    pub user_id: String,
    pub login_time: i64,
    pub ip_address: String,
    pub user_agent: String,
    pub status: LoginStatus,
}

impl LoginHistory {
    pub fn new(
        id: i64,
        user_id: String,
        ip_address: String,
        user_agent: String,
        status: LoginStatus,
    ) -> Self {
        Self {
            id,
            user_id,
            login_time: chrono::Utc::now().timestamp(),
            ip_address,
            user_agent,
            status,
        }
    }
}

/// JWT Claims
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,        // user_id
    pub username: String,
    pub role: String,
    pub exp: i64,          // expiration time
    pub iat: i64,          // issued at
}
