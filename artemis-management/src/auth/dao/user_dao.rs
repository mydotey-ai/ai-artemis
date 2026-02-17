use crate::auth::model::{User, UserRole, UserStatus};
use sea_orm::{DatabaseConnection, Statement, ConnectionTrait};
use sea_orm::sea_query::Value;

pub struct UserDao {
    conn: DatabaseConnection,
}

impl UserDao {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    /// 插入用户
    pub async fn insert_user(&self, user: &User) -> anyhow::Result<()> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            INSERT INTO auth_users (user_id, username, email, description, password_hash, role, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            vec![
                Value::from(&user.user_id),
                Value::from(&user.username),
                Value::from(user.email.as_deref().unwrap_or("")),
                Value::from(user.description.as_deref().unwrap_or("")),
                Value::from(&user.password_hash),
                Value::from(user.role.as_str()),
                Value::from(user.status.as_str()),
                Value::from(user.created_at),
                Value::from(user.updated_at),
            ],
        );

        self.conn.execute(stmt).await?;
        Ok(())
    }

    /// 更新用户
    pub async fn update_user(&self, user: &User) -> anyhow::Result<()> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            UPDATE auth_users
            SET email = ?, description = ?, password_hash = ?, role = ?, status = ?, updated_at = ?
            WHERE user_id = ?
            "#,
            vec![
                Value::from(user.email.as_deref().unwrap_or("")),
                Value::from(user.description.as_deref().unwrap_or("")),
                Value::from(&user.password_hash),
                Value::from(user.role.as_str()),
                Value::from(user.status.as_str()),
                Value::from(user.updated_at),
                Value::from(&user.user_id),
            ],
        );

        self.conn.execute(stmt).await?;
        Ok(())
    }

    /// 删除用户
    pub async fn delete_user(&self, user_id: &str) -> anyhow::Result<()> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "DELETE FROM auth_users WHERE user_id = ?",
            vec![Value::from(user_id)],
        );

        self.conn.execute(stmt).await?;
        Ok(())
    }

    /// 根据 user_id 获取用户
    pub async fn get_user(&self, user_id: &str) -> anyhow::Result<Option<User>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "SELECT user_id, username, email, description, password_hash, role, status, created_at, updated_at FROM auth_users WHERE user_id = ?",
            vec![Value::from(user_id)],
        );

        let result = self.conn.query_one(stmt).await?;

        if let Some(row) = result {
            Ok(Some(self.row_to_user(row)?))
        } else {
            Ok(None)
        }
    }

    /// 根据 username 获取用户
    pub async fn get_user_by_username(&self, username: &str) -> anyhow::Result<Option<User>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "SELECT user_id, username, email, description, password_hash, role, status, created_at, updated_at FROM auth_users WHERE username = ?",
            vec![Value::from(username)],
        );

        let result = self.conn.query_one(stmt).await?;

        if let Some(row) = result {
            Ok(Some(self.row_to_user(row)?))
        } else {
            Ok(None)
        }
    }

    /// 列出所有用户
    pub async fn list_users(&self) -> anyhow::Result<Vec<User>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "SELECT user_id, username, email, description, password_hash, role, status, created_at, updated_at FROM auth_users ORDER BY created_at DESC",
            vec![],
        );

        let results = self.conn.query_all(stmt).await?;

        let mut users = Vec::new();
        for row in results {
            users.push(self.row_to_user(row)?);
        }

        Ok(users)
    }

    /// 按角色列出用户
    pub async fn list_users_by_role(&self, role: UserRole) -> anyhow::Result<Vec<User>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "SELECT user_id, username, email, description, password_hash, role, status, created_at, updated_at FROM auth_users WHERE role = ? ORDER BY created_at DESC",
            vec![Value::from(role.as_str())],
        );

        let results = self.conn.query_all(stmt).await?;

        let mut users = Vec::new();
        for row in results {
            users.push(self.row_to_user(row)?);
        }

        Ok(users)
    }

    /// 按状态列出用户
    pub async fn list_users_by_status(&self, status: UserStatus) -> anyhow::Result<Vec<User>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "SELECT user_id, username, email, description, password_hash, role, status, created_at, updated_at FROM auth_users WHERE status = ? ORDER BY created_at DESC",
            vec![Value::from(status.as_str())],
        );

        let results = self.conn.query_all(stmt).await?;

        let mut users = Vec::new();
        for row in results {
            users.push(self.row_to_user(row)?);
        }

        Ok(users)
    }

    /// 将数据库行转换为 User 对象
    fn row_to_user(&self, row: sea_orm::QueryResult) -> anyhow::Result<User> {
        let user_id: String = row.try_get("", "user_id")?;
        let username: String = row.try_get("", "username")?;
        let email_str: String = row.try_get("", "email").unwrap_or_default();
        let email = if email_str.is_empty() { None } else { Some(email_str) };
        let description_str: String = row.try_get("", "description").unwrap_or_default();
        let description = if description_str.is_empty() { None } else { Some(description_str) };
        let password_hash: String = row.try_get("", "password_hash")?;
        let role_str: String = row.try_get("", "role")?;
        let status_str: String = row.try_get("", "status")?;
        let created_at: i64 = row.try_get("", "created_at")?;
        let updated_at: i64 = row.try_get("", "updated_at")?;

        let role = UserRole::from_str(&role_str)
            .ok_or_else(|| anyhow::anyhow!("Invalid role: {}", role_str))?;
        let status = UserStatus::from_str(&status_str)
            .ok_or_else(|| anyhow::anyhow!("Invalid status: {}", status_str))?;

        Ok(User {
            user_id,
            username,
            email,
            description,
            password_hash,
            role,
            status,
            created_at,
            updated_at,
        })
    }
}
