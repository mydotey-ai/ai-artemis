use sqlx::{Any, Pool};
use sqlx::any::AnyPoolOptions;
use std::time::Duration;

/// 数据库连接管理器
///
/// 支持 SQLite 和 MySQL 数据库
#[derive(Clone)]
pub struct Database {
    pool: Pool<Any>,
    db_type: DatabaseType,
}

/// 数据库类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    SQLite,
    MySQL,
}

impl Database {
    /// 创建新的数据库连接
    ///
    /// # Arguments
    /// * `database_url` - 数据库连接 URL
    ///   - SQLite: "sqlite://artemis.db" 或 "sqlite::memory:"
    ///   - MySQL: "mysql://user:password@localhost:3306/artemis"
    /// * `max_connections` - 最大连接数 (默认 10)
    pub async fn new(database_url: &str, max_connections: u32) -> anyhow::Result<Self> {
        // 检测数据库类型
        let db_type = Self::detect_db_type(database_url)?;

        tracing::info!(
            "Initializing database: {} (type: {:?})",
            Self::sanitize_url(database_url),
            db_type
        );

        // 创建连接池 (使用 Any 类型支持多种数据库)
        let pool = AnyPoolOptions::new()
            .max_connections(max_connections)
            .acquire_timeout(Duration::from_secs(5))
            .connect(database_url)
            .await?;

        Ok(Self { pool, db_type })
    }

    /// 检测数据库类型
    fn detect_db_type(url: &str) -> anyhow::Result<DatabaseType> {
        if url.starts_with("sqlite:") {
            Ok(DatabaseType::SQLite)
        } else if url.starts_with("mysql:") {
            Ok(DatabaseType::MySQL)
        } else {
            Err(anyhow::anyhow!(
                "Unsupported database URL format: {}. Expected 'sqlite:' or 'mysql:'",
                url
            ))
        }
    }

    /// 净化 URL (隐藏密码)
    fn sanitize_url(url: &str) -> String {
        if let Some(at_pos) = url.find('@')
            && let Some(colon_pos) = url[..at_pos].rfind(':')
        {
            let mut sanitized = url.to_string();
            sanitized.replace_range(colon_pos + 1..at_pos, "****");
            return sanitized;
        }
        url.to_string()
    }

    /// 获取连接池引用
    pub fn pool(&self) -> &Pool<Any> {
        &self.pool
    }

    /// 获取数据库类型
    pub fn db_type(&self) -> DatabaseType {
        self.db_type
    }

    /// 健康检查
    pub async fn health_check(&self) -> anyhow::Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }

    /// 运行数据库迁移
    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        tracing::info!("Running database migrations for {:?}", self.db_type);

        match self.db_type {
            DatabaseType::SQLite => {
                // SQLite 使用现有的迁移文件
                sqlx::migrate!("./migrations")
                    .run(&self.pool)
                    .await?;
            }
            DatabaseType::MySQL => {
                // MySQL 使用相同的迁移文件 (SQLx 会自动处理方言差异)
                sqlx::migrate!("./migrations")
                    .run(&self.pool)
                    .await?;
            }
        }

        tracing::info!("Database migrations completed");
        Ok(())
    }

    /// 关闭连接池
    pub async fn close(&self) {
        self.pool.close().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Pool<Any> 在单元测试中有已知问题
    // 实际数据库连接测试在集成测试中进行
    // 参见: test-persistence.sh

    #[test]
    fn test_detect_db_type() {
        assert_eq!(
            Database::detect_db_type("sqlite://test.db").unwrap(),
            DatabaseType::SQLite
        );
        assert_eq!(
            Database::detect_db_type("mysql://user:pass@localhost/db").unwrap(),
            DatabaseType::MySQL
        );
        assert!(Database::detect_db_type("postgres://localhost").is_err());
    }

    #[test]
    fn test_sanitize_url() {
        assert_eq!(
            Database::sanitize_url("mysql://user:password@localhost/db"),
            "mysql://user:****@localhost/db"
        );
        assert_eq!(
            Database::sanitize_url("sqlite://test.db"),
            "sqlite://test.db"
        );
    }
}
