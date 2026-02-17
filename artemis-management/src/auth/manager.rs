use crate::auth::model::{User, UserRole, UserStatus, Session, LoginHistory, LoginStatus, JwtClaims};
use crate::auth::dao::{UserDao, SessionDao};
use crate::db::Database;
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};

pub struct AuthManager {
    // 内存存储
    users: Arc<DashMap<String, User>>,                    // user_id -> User
    username_map: Arc<DashMap<String, String>>,           // username -> user_id
    sessions: Arc<DashMap<String, Session>>,              // session_id -> Session
    token_map: Arc<DashMap<String, String>>,              // token -> session_id
    login_history: Arc<DashMap<i64, LoginHistory>>,      // history_id -> LoginHistory

    // ID 生成
    next_history_id: Arc<AtomicI64>,

    // JWT 配置
    jwt_secret: String,
    jwt_expiry_seconds: i64,

    // 持久化
    database: Option<Arc<Database>>,
}

impl AuthManager {
    /// 创建新的 AuthManager (无持久化)
    pub fn new() -> Self {
        Self::with_database(None, "artemis-default-secret-change-in-production".to_string())
    }

    /// 创建带数据库持久化的 AuthManager
    pub fn with_database(database: Option<Arc<Database>>, jwt_secret: String) -> Self {
        Self {
            users: Arc::new(DashMap::new()),
            username_map: Arc::new(DashMap::new()),
            sessions: Arc::new(DashMap::new()),
            token_map: Arc::new(DashMap::new()),
            login_history: Arc::new(DashMap::new()),
            next_history_id: Arc::new(AtomicI64::new(1)),
            jwt_secret,
            jwt_expiry_seconds: std::env::var("JWT_EXPIRY_SECONDS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(3600), // 默认1小时
            database,
        }
    }

    /// 从数据库加载数据(异步方法)
    pub async fn load_from_database(&self) -> anyhow::Result<()> {
        if let Some(database) = &self.database {
            let user_dao = UserDao::new(database.conn().clone());
            let session_dao = SessionDao::new(database.conn().clone());

            // 加载用户
            let users = user_dao.list_users().await?;
            for user in users {
                self.username_map.insert(user.username.clone(), user.user_id.clone());
                self.users.insert(user.user_id.clone(), user);
            }

            // 加载会话
            let current_time = chrono::Utc::now().timestamp();
            for user_ref in self.users.iter() {
                let sessions = session_dao.list_sessions_by_user(&user_ref.user_id).await?;
                for session in sessions {
                    // 过滤过期会话
                    if session.expires_at > current_time {
                        if let Some(token) = &session.token {
                            self.token_map.insert(token.clone(), session.session_id.clone());
                        }
                        self.sessions.insert(session.session_id.clone(), session);
                    }
                }
            }

            tracing::info!("Loaded {} users and {} active sessions from database",
                self.users.len(), self.sessions.len());
        }

        Ok(())
    }

    /// 用户认证
    pub fn authenticate(
        &self,
        username: &str,
        password: &str,
        ip: Option<String>,
        user_agent: Option<String>,
    ) -> Result<String, String> {
        // 获取用户
        let user_id = self.username_map.get(username)
            .ok_or_else(|| "Invalid username or password".to_string())?;

        let user = self.users.get(user_id.value())
            .ok_or_else(|| "Invalid username or password".to_string())?;

        // 检查用户状态
        if user.status != UserStatus::Active {
            self.record_login_history(&user.user_id, ip.clone().unwrap_or_default(),
                user_agent.clone().unwrap_or_default(), LoginStatus::Failed);
            return Err("User account is inactive".to_string());
        }

        // 验证密码
        if !self.verify_password(password, &user.password_hash) {
            self.record_login_history(&user.user_id, ip.clone().unwrap_or_default(),
                user_agent.clone().unwrap_or_default(), LoginStatus::Failed);
            return Err("Invalid username or password".to_string());
        }

        // 生成 JWT token
        let token = self.generate_jwt_token(&user)?;

        // 创建会话
        let session = Session::new(
            user.user_id.clone(),
            token.clone(),
            ip.clone(),
            user_agent.clone(),
            self.jwt_expiry_seconds,
        );

        self.token_map.insert(token.clone(), session.session_id.clone());
        self.sessions.insert(session.session_id.clone(), session.clone());

        // 记录登录历史
        self.record_login_history(&user.user_id, ip.unwrap_or_default(),
            user_agent.unwrap_or_default(), LoginStatus::Success);

        // 持久化会话
        if let Some(db) = &self.database {
            let session_dao = SessionDao::new(db.conn().clone());
            let session_clone = session.clone();
            tokio::spawn(async move {
                if let Err(e) = session_dao.insert_session(&session_clone).await {
                    tracing::error!("Failed to persist session: {:?}", e);
                }
            });
        }

        Ok(token)
    }

    /// 验证 token
    pub fn validate_token(&self, token: &str) -> Result<Session, String> {
        // 从 token 获取会话
        let session_id = self.token_map.get(token)
            .ok_or_else(|| "Invalid or expired token".to_string())?;

        let session = self.sessions.get(session_id.value())
            .ok_or_else(|| "Invalid or expired token".to_string())?;

        // 检查过期
        if session.is_expired() {
            self.sessions.remove(session_id.value());
            self.token_map.remove(token);
            return Err("Token expired".to_string());
        }

        // 验证 JWT
        self.verify_jwt_token(token)?;

        Ok(session.clone())
    }

    /// 登出
    pub fn logout(&self, token: &str) -> Result<(), String> {
        let session_id = self.token_map.get(token)
            .ok_or_else(|| "Invalid token".to_string())?;

        let session_id_value = session_id.value().clone();
        drop(session_id);

        self.sessions.remove(&session_id_value);
        self.token_map.remove(token);

        // 持久化删除
        if let Some(db) = &self.database {
            let session_dao = SessionDao::new(db.conn().clone());
            let session_id_clone = session_id_value.clone();
            tokio::spawn(async move {
                if let Err(e) = session_dao.delete_session(&session_id_clone).await {
                    tracing::error!("Failed to delete session from database: {:?}", e);
                }
            });
        }

        Ok(())
    }

    /// 刷新 token
    pub fn refresh_token(&self, old_token: &str) -> Result<String, String> {
        // 验证旧 token
        let session = self.validate_token(old_token)?;

        // 获取用户
        let user = self.users.get(&session.user_id)
            .ok_or_else(|| "User not found".to_string())?;

        // 生成新 token
        let new_token = self.generate_jwt_token(&user)?;

        // 更新会话
        let mut session_mut = self.sessions.get_mut(&session.session_id)
            .ok_or_else(|| "Session not found".to_string())?;

        session_mut.token = Some(new_token.clone());
        session_mut.expires_at = chrono::Utc::now().timestamp() + self.jwt_expiry_seconds;

        // 更新映射
        self.token_map.remove(old_token);
        self.token_map.insert(new_token.clone(), session.session_id.clone());

        Ok(new_token)
    }

    /// 创建用户
    pub fn create_user(
        &self,
        username: &str,
        email: Option<String>,
        description: Option<String>,
        password: &str,
        role: UserRole,
    ) -> Result<User, String> {
        // 检查用户名是否已存在
        if self.username_map.contains_key(username) {
            return Err(format!("Username '{}' already exists", username));
        }

        // 哈希密码
        let password_hash = self.hash_password(password)?;

        // 创建用户
        let user = User::new(username.to_string(), email, description, password_hash, role);

        // 存储
        self.username_map.insert(user.username.clone(), user.user_id.clone());
        self.users.insert(user.user_id.clone(), user.clone());

        // 持久化
        if let Some(db) = &self.database {
            let user_dao = UserDao::new(db.conn().clone());
            let user_clone = user.clone();
            tokio::spawn(async move {
                if let Err(e) = user_dao.insert_user(&user_clone).await {
                    tracing::error!("Failed to persist user: {:?}", e);
                }
            });
        }

        Ok(user)
    }

    /// 更新用户
    pub fn update_user(
        &self,
        user_id: &str,
        email: Option<String>,
        description: Option<String>,
        role: Option<UserRole>,
    ) -> Result<User, String> {
        let mut user = self.users.get_mut(user_id)
            .ok_or_else(|| "User not found".to_string())?;

        if let Some(e) = email {
            user.email = Some(e);
        }
        if let Some(d) = description {
            user.description = Some(d);
        }
        if let Some(r) = role {
            user.role = r;
        }
        user.updated_at = chrono::Utc::now().timestamp();

        let updated_user = user.clone();
        drop(user);

        // 持久化
        if let Some(db) = &self.database {
            let user_dao = UserDao::new(db.conn().clone());
            let user_clone = updated_user.clone();
            tokio::spawn(async move {
                if let Err(e) = user_dao.update_user(&user_clone).await {
                    tracing::error!("Failed to update user in database: {:?}", e);
                }
            });
        }

        Ok(updated_user)
    }

    /// 删除用户
    pub fn delete_user(&self, user_id: &str) -> Result<(), String> {
        let user = self.users.get(user_id)
            .ok_or_else(|| "User not found".to_string())?;

        let username = user.username.clone();
        drop(user);

        self.username_map.remove(&username);
        self.users.remove(user_id);

        // 删除用户的所有会话
        let session_ids: Vec<String> = self.sessions.iter()
            .filter(|s| s.user_id == user_id)
            .map(|s| s.session_id.clone())
            .collect();

        for session_id in session_ids {
            if let Some(session) = self.sessions.remove(&session_id) {
                if let Some(token) = &session.1.token {
                    self.token_map.remove(token);
                }
            }
        }

        // 持久化删除
        if let Some(db) = &self.database {
            let user_dao = UserDao::new(db.conn().clone());
            let session_dao = SessionDao::new(db.conn().clone());
            let user_id_clone = user_id.to_string();
            tokio::spawn(async move {
                if let Err(e) = user_dao.delete_user(&user_id_clone).await {
                    tracing::error!("Failed to delete user from database: {:?}", e);
                }
                if let Err(e) = session_dao.delete_user_sessions(&user_id_clone).await {
                    tracing::error!("Failed to delete user sessions from database: {:?}", e);
                }
            });
        }

        Ok(())
    }

    /// 获取用户
    pub fn get_user(&self, user_id: &str) -> Option<User> {
        self.users.get(user_id).map(|u| u.clone())
    }

    /// 根据用户名获取用户
    pub fn get_user_by_username(&self, username: &str) -> Option<User> {
        let user_id = self.username_map.get(username)?;
        self.users.get(user_id.value()).map(|u| u.clone())
    }

    /// 列出所有用户
    pub fn list_users(&self) -> Vec<User> {
        self.users.iter().map(|entry| entry.value().clone()).collect()
    }

    /// 修改密码
    pub fn change_password(
        &self,
        user_id: &str,
        old_password: &str,
        new_password: &str,
    ) -> Result<(), String> {
        let mut user = self.users.get_mut(user_id)
            .ok_or_else(|| "User not found".to_string())?;

        // 验证旧密码
        if !self.verify_password(old_password, &user.password_hash) {
            return Err("Old password is incorrect".to_string());
        }

        // 设置新密码
        user.password_hash = self.hash_password(new_password)?;
        user.updated_at = chrono::Utc::now().timestamp();

        let updated_user = user.clone();
        drop(user);

        // 撤销所有会话
        self.revoke_all_user_sessions(user_id)?;

        // 持久化
        if let Some(db) = &self.database {
            let user_dao = UserDao::new(db.conn().clone());
            tokio::spawn(async move {
                if let Err(e) = user_dao.update_user(&updated_user).await {
                    tracing::error!("Failed to update user password in database: {:?}", e);
                }
            });
        }

        Ok(())
    }

    /// 重置密码 (管理员操作)
    pub fn reset_password(&self, user_id: &str, new_password: &str) -> Result<(), String> {
        let mut user = self.users.get_mut(user_id)
            .ok_or_else(|| "User not found".to_string())?;

        user.password_hash = self.hash_password(new_password)?;
        user.updated_at = chrono::Utc::now().timestamp();

        let updated_user = user.clone();
        drop(user);

        // 撤销所有会话
        self.revoke_all_user_sessions(user_id)?;

        // 持久化
        if let Some(db) = &self.database {
            let user_dao = UserDao::new(db.conn().clone());
            tokio::spawn(async move {
                if let Err(e) = user_dao.update_user(&updated_user).await {
                    tracing::error!("Failed to reset user password in database: {:?}", e);
                }
            });
        }

        Ok(())
    }

    /// 修改用户状态
    pub fn change_user_status(&self, user_id: &str, status: UserStatus) -> Result<User, String> {
        let mut user = self.users.get_mut(user_id)
            .ok_or_else(|| "User not found".to_string())?;

        user.status = status.clone();
        user.updated_at = chrono::Utc::now().timestamp();

        let updated_user = user.clone();
        drop(user);

        // 如果设置为 Inactive,撤销所有会话
        if status == UserStatus::Inactive {
            self.revoke_all_user_sessions(user_id)?;
        }

        // 持久化
        if let Some(db) = &self.database {
            let user_dao = UserDao::new(db.conn().clone());
            let user_clone = updated_user.clone();
            tokio::spawn(async move {
                if let Err(e) = user_dao.update_user(&user_clone).await {
                    tracing::error!("Failed to update user status in database: {:?}", e);
                }
            });
        }

        Ok(updated_user)
    }

    /// 列出用户的会话
    pub fn list_user_sessions(&self, user_id: &str) -> Vec<Session> {
        self.sessions.iter()
            .filter(|s| s.user_id == user_id)
            .map(|s| {
                let mut session = s.value().clone();
                session.token = None; // 不返回 token
                session
            })
            .collect()
    }

    /// 撤销会话
    pub fn revoke_session(&self, session_id: &str) -> Result<(), String> {
        let session = self.sessions.remove(session_id)
            .ok_or_else(|| "Session not found".to_string())?;

        if let Some(token) = &session.1.token {
            self.token_map.remove(token);
        }

        // 持久化删除
        if let Some(db) = &self.database {
            let session_dao = SessionDao::new(db.conn().clone());
            let session_id_clone = session_id.to_string();
            tokio::spawn(async move {
                if let Err(e) = session_dao.delete_session(&session_id_clone).await {
                    tracing::error!("Failed to delete session from database: {:?}", e);
                }
            });
        }

        Ok(())
    }

    /// 撤销用户的所有会话
    pub fn revoke_all_user_sessions(&self, user_id: &str) -> Result<usize, String> {
        let session_ids: Vec<String> = self.sessions.iter()
            .filter(|s| s.user_id == user_id)
            .map(|s| s.session_id.clone())
            .collect();

        let count = session_ids.len();

        for session_id in session_ids {
            if let Some(session) = self.sessions.remove(&session_id) {
                if let Some(token) = &session.1.token {
                    self.token_map.remove(token);
                }
            }
        }

        // 持久化删除
        if let Some(db) = &self.database {
            let session_dao = SessionDao::new(db.conn().clone());
            let user_id_clone = user_id.to_string();
            tokio::spawn(async move {
                if let Err(e) = session_dao.delete_user_sessions(&user_id_clone).await {
                    tracing::error!("Failed to delete user sessions from database: {:?}", e);
                }
            });
        }

        Ok(count)
    }

    /// 检查权限
    pub fn check_permission(&self, user_id: &str, resource: &str, action: &str) -> bool {
        let user = match self.users.get(user_id) {
            Some(u) => u,
            None => return false,
        };

        match user.role {
            UserRole::Admin => true,  // Admin 有全部权限
            UserRole::Operator => match (resource, action) {
                ("services", _) | ("instances", _) | ("routing", _) => true,
                ("cluster", "read") | ("audit", "read") => true,
                ("auth", "read") => true, // 可以查看自己的信息
                _ => false,
            },
            UserRole::Viewer => action == "read",  // Viewer 只有读权限
        }
    }

    /// 获取用户权限列表
    pub fn get_user_permissions(&self, user_id: &str) -> Vec<String> {
        let user = match self.users.get(user_id) {
            Some(u) => u,
            None => return vec![],
        };

        match user.role {
            UserRole::Admin => vec!["*:*".to_string()],
            UserRole::Operator => vec![
                "services:*".to_string(),
                "instances:*".to_string(),
                "routing:*".to_string(),
                "cluster:read".to_string(),
                "audit:read".to_string(),
                "auth:read".to_string(),
            ],
            UserRole::Viewer => vec!["*:read".to_string()],
        }
    }

    /// 获取登录历史
    pub fn get_login_history(&self, user_id: &str, limit: usize) -> Vec<LoginHistory> {
        let mut history: Vec<LoginHistory> = self.login_history.iter()
            .filter(|h| h.user_id == user_id)
            .map(|h| h.value().clone())
            .collect();

        history.sort_by(|a, b| b.login_time.cmp(&a.login_time));
        history.truncate(limit);
        history
    }

    /// 记录登录历史
    fn record_login_history(&self, user_id: &str, ip: String, user_agent: String, status: LoginStatus) {
        let id = self.next_history_id.fetch_add(1, Ordering::SeqCst);
        let history = LoginHistory::new(id, user_id.to_string(), ip, user_agent, status);

        self.login_history.insert(id, history.clone());

        // 持久化
        if let Some(db) = &self.database {
            let session_dao = SessionDao::new(db.conn().clone());
            tokio::spawn(async move {
                if let Err(e) = session_dao.insert_login_history(&history).await {
                    tracing::error!("Failed to persist login history: {:?}", e);
                }
            });
        }
    }

    /// 生成 JWT token
    fn generate_jwt_token(&self, user: &User) -> Result<String, String> {
        let now = chrono::Utc::now().timestamp();
        let claims = JwtClaims {
            sub: user.user_id.clone(),
            username: user.username.clone(),
            role: user.role.as_str().to_string(),
            exp: now + self.jwt_expiry_seconds,
            iat: now,
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.jwt_secret.as_bytes()),
        )
        .map_err(|e| format!("Failed to generate JWT token: {}", e))
    }

    /// 验证 JWT token
    fn verify_jwt_token(&self, token: &str) -> Result<JwtClaims, String> {
        decode::<JwtClaims>(
            token,
            &DecodingKey::from_secret(self.jwt_secret.as_bytes()),
            &Validation::default(),
        )
        .map(|data| data.claims)
        .map_err(|e| format!("Invalid JWT token: {}", e))
    }

    /// 哈希密码
    fn hash_password(&self, password: &str) -> Result<String, String> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| format!("Failed to hash password: {}", e))
    }

    /// 验证密码
    fn verify_password(&self, password: &str, hash: &str) -> bool {
        bcrypt::verify(password, hash).unwrap_or(false)
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}
