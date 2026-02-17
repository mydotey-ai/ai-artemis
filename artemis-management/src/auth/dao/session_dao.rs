use crate::auth::model::{LoginHistory, LoginStatus, Session};
use sea_orm::sea_query::Value;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use std::str::FromStr;

pub struct SessionDao {
    conn: DatabaseConnection,
}

impl SessionDao {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    /// 插入会话
    pub async fn insert_session(&self, session: &Session) -> anyhow::Result<()> {
        let token = session.token.as_deref().unwrap_or("");

        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            INSERT INTO auth_sessions (session_id, user_id, token, ip_address, user_agent, created_at, expires_at, last_activity)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            vec![
                Value::from(&session.session_id),
                Value::from(&session.user_id),
                Value::from(token),
                Value::from(session.ip_address.as_deref().unwrap_or("")),
                Value::from(session.user_agent.as_deref().unwrap_or("")),
                Value::from(session.created_at),
                Value::from(session.expires_at),
                Value::from(session.last_activity),
            ],
        );

        self.conn.execute(stmt).await?;
        Ok(())
    }

    /// 更新最后活动时间
    pub async fn update_last_activity(
        &self,
        session_id: &str,
        timestamp: i64,
    ) -> anyhow::Result<()> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "UPDATE auth_sessions SET last_activity = ? WHERE session_id = ?",
            vec![Value::from(timestamp), Value::from(session_id)],
        );

        self.conn.execute(stmt).await?;
        Ok(())
    }

    /// 删除会话
    pub async fn delete_session(&self, session_id: &str) -> anyhow::Result<()> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "DELETE FROM auth_sessions WHERE session_id = ?",
            vec![Value::from(session_id)],
        );

        self.conn.execute(stmt).await?;
        Ok(())
    }

    /// 根据 session_id 获取会话
    pub async fn get_session(&self, session_id: &str) -> anyhow::Result<Option<Session>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "SELECT session_id, user_id, token, ip_address, user_agent, created_at, expires_at, last_activity FROM auth_sessions WHERE session_id = ?",
            vec![Value::from(session_id)],
        );

        let result = self.conn.query_one(stmt).await?;

        if let Some(row) = result { Ok(Some(self.row_to_session(row)?)) } else { Ok(None) }
    }

    /// 根据 token 获取会话
    pub async fn get_session_by_token(&self, token: &str) -> anyhow::Result<Option<Session>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "SELECT session_id, user_id, token, ip_address, user_agent, created_at, expires_at, last_activity FROM auth_sessions WHERE token = ?",
            vec![Value::from(token)],
        );

        let result = self.conn.query_one(stmt).await?;

        if let Some(row) = result { Ok(Some(self.row_to_session(row)?)) } else { Ok(None) }
    }

    /// 列出用户的所有会话
    pub async fn list_sessions_by_user(&self, user_id: &str) -> anyhow::Result<Vec<Session>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "SELECT session_id, user_id, token, ip_address, user_agent, created_at, expires_at, last_activity FROM auth_sessions WHERE user_id = ? ORDER BY created_at DESC",
            vec![Value::from(user_id)],
        );

        let results = self.conn.query_all(stmt).await?;

        let mut sessions = Vec::new();
        for row in results {
            sessions.push(self.row_to_session(row)?);
        }

        Ok(sessions)
    }

    /// 删除过期会话
    pub async fn delete_expired_sessions(&self, current_time: i64) -> anyhow::Result<u64> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "DELETE FROM auth_sessions WHERE expires_at < ?",
            vec![Value::from(current_time)],
        );

        let result = self.conn.execute(stmt).await?;
        Ok(result.rows_affected())
    }

    /// 删除用户的所有会话
    pub async fn delete_user_sessions(&self, user_id: &str) -> anyhow::Result<u64> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "DELETE FROM auth_sessions WHERE user_id = ?",
            vec![Value::from(user_id)],
        );

        let result = self.conn.execute(stmt).await?;
        Ok(result.rows_affected())
    }

    /// 插入登录历史
    pub async fn insert_login_history(&self, history: &LoginHistory) -> anyhow::Result<()> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            INSERT INTO auth_login_history (user_id, login_time, ip_address, user_agent, status)
            VALUES (?, ?, ?, ?, ?)
            "#,
            vec![
                Value::from(&history.user_id),
                Value::from(history.login_time),
                Value::from(&history.ip_address),
                Value::from(&history.user_agent),
                Value::from(history.status.as_str()),
            ],
        );

        self.conn.execute(stmt).await?;
        Ok(())
    }

    /// 列出用户的登录历史
    pub async fn list_login_history(
        &self,
        user_id: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<LoginHistory>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "SELECT id, user_id, login_time, ip_address, user_agent, status FROM auth_login_history WHERE user_id = ? ORDER BY login_time DESC LIMIT ?",
            vec![Value::from(user_id), Value::from(limit as i32)],
        );

        let results = self.conn.query_all(stmt).await?;

        let mut history = Vec::new();
        for row in results {
            history.push(self.row_to_login_history(row)?);
        }

        Ok(history)
    }

    /// 将数据库行转换为 Session 对象
    fn row_to_session(&self, row: sea_orm::QueryResult) -> anyhow::Result<Session> {
        let session_id: String = row.try_get("", "session_id")?;
        let user_id: String = row.try_get("", "user_id")?;
        let token: String = row.try_get("", "token")?;
        let ip_str: String = row.try_get("", "ip_address")?;
        let ua_str: String = row.try_get("", "user_agent")?;
        let created_at: i64 = row.try_get("", "created_at")?;
        let expires_at: i64 = row.try_get("", "expires_at")?;
        let last_activity: i64 = row.try_get("", "last_activity")?;

        let ip_address = if ip_str.is_empty() { None } else { Some(ip_str) };
        let user_agent = if ua_str.is_empty() { None } else { Some(ua_str) };

        Ok(Session {
            session_id,
            user_id,
            token: Some(token),
            ip_address,
            user_agent,
            created_at,
            expires_at,
            last_activity,
        })
    }

    /// 将数据库行转换为 LoginHistory 对象
    fn row_to_login_history(&self, row: sea_orm::QueryResult) -> anyhow::Result<LoginHistory> {
        let id: i64 = row.try_get("", "id")?;
        let user_id: String = row.try_get("", "user_id")?;
        let login_time: i64 = row.try_get("", "login_time")?;
        let ip_address: String = row.try_get("", "ip_address")?;
        let user_agent: String = row.try_get("", "user_agent")?;
        let status_str: String = row.try_get("", "status")?;

        let status = LoginStatus::from_str(&status_str).map_err(|e| anyhow::anyhow!(e))?;

        Ok(LoginHistory { id, user_id, login_time, ip_address, user_agent, status })
    }
}
