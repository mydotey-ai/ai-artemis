use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use sqlx::migrate::MigrateDatabase;
use std::time::Duration;

/// 数据库连接管理器
#[derive(Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// 创建新的数据库连接
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        // 如果数据库不存在,创建它
        if !sqlx::Sqlite::database_exists(database_url).await? {
            tracing::info!("Creating database: {}", database_url);
            sqlx::Sqlite::create_database(database_url).await?;
        }

        // 创建连接池
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(5))
            .connect(database_url)
            .await?;

        Ok(Self { pool })
    }

    /// 获取连接池引用
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
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
        tracing::info!("Running database migrations");
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await?;
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

    #[tokio::test]
    async fn test_database_creation() {
        let db = Database::new("sqlite::memory:").await.unwrap();
        assert!(db.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_migrations() {
        let db = Database::new("sqlite::memory:").await.unwrap();
        assert!(db.run_migrations().await.is_ok());

        // 验证表是否创建
        let result: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM sqlite_master WHERE type='table'")
            .fetch_one(db.pool())
            .await
            .unwrap();

        // 应该有12张表 + 1张 _sqlx_migrations 表
        assert!(result.0 >= 12);
    }
}
