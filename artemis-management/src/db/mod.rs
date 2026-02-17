use sea_orm::{Database as SeaDatabase, DatabaseConnection, ConnectOptions, DbErr};
use std::time::Duration;

/// 数据库连接管理器
///
/// 使用SeaORM,原生支持SQLite和MySQL的运行时切换
#[derive(Clone)]
pub struct Database {
    conn: DatabaseConnection,
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

        // 配置连接选项
        let mut opt = ConnectOptions::new(database_url.to_owned());
        opt.max_connections(max_connections)
            .connect_timeout(Duration::from_secs(5))
            .acquire_timeout(Duration::from_secs(5));

        // 连接数据库 - SeaORM自动处理不同数据库类型!
        let conn = SeaDatabase::connect(opt).await?;

        Ok(Self { conn, db_type })
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

    /// 获取数据库连接引用
    pub fn conn(&self) -> &DatabaseConnection {
        &self.conn
    }

    /// 获取数据库类型
    pub fn db_type(&self) -> DatabaseType {
        self.db_type
    }

    /// 健康检查
    pub async fn health_check(&self) -> anyhow::Result<()> {
        // SeaORM的ping方法
        self.conn.ping().await?;
        Ok(())
    }

    /// 运行数据库迁移
    ///
    /// 执行 migrations/ 目录下的所有 SQL 迁移文件
    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        tracing::info!("Running database migrations for {:?}", self.db_type);

        // 迁移1: 初始Schema (实例操作、服务分组、路由规则等)
        let migration_001 = include_str!("../../migrations/001_initial_schema.sql");
        self.execute_migration("001_initial_schema", migration_001).await?;

        // 迁移2: 认证系统 (用户、会话、登录历史)
        let migration_002 = include_str!("../../migrations/002_auth_schema.sql");
        self.execute_migration("002_auth_schema", migration_002).await?;

        tracing::info!("All database migrations completed successfully");

        Ok(())
    }

    /// 执行单个迁移
    async fn execute_migration(&self, name: &str, sql: &str) -> anyhow::Result<()> {
        use sea_orm::ConnectionTrait;

        tracing::debug!("Executing migration: {}", name);

        // 按分号分割SQL语句并逐个执行
        for statement in sql.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() && !trimmed.starts_with("--") {
                self.conn.execute_unprepared(trimmed).await
                    .map_err(|e| anyhow::anyhow!("Failed to execute migration {}: {}", name, e))?;
            }
        }

        tracing::debug!("Migration {} completed", name);

        Ok(())
    }

    /// 关闭连接池
    pub async fn close(&self) -> Result<(), DbErr> {
        self.conn.clone().close().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========== DatabaseType 测试 ==========

    #[test]
    fn test_database_type_debug() {
        let db_type = DatabaseType::SQLite;
        let debug_str = format!("{:?}", db_type);
        assert!(debug_str.contains("SQLite"));
    }

    #[test]
    fn test_database_type_clone() {
        let db_type = DatabaseType::MySQL;
        let cloned = db_type;
        assert_eq!(cloned, DatabaseType::MySQL);
    }

    #[test]
    fn test_database_type_equality() {
        assert_eq!(DatabaseType::SQLite, DatabaseType::SQLite);
        assert_eq!(DatabaseType::MySQL, DatabaseType::MySQL);
        assert_ne!(DatabaseType::SQLite, DatabaseType::MySQL);
    }

    // ========== detect_db_type 测试 ==========

    #[test]
    fn test_detect_db_type_sqlite() {
        assert_eq!(
            Database::detect_db_type("sqlite://test.db").unwrap(),
            DatabaseType::SQLite
        );
        assert_eq!(
            Database::detect_db_type("sqlite::memory:").unwrap(),
            DatabaseType::SQLite
        );
    }

    #[test]
    fn test_detect_db_type_mysql() {
        assert_eq!(
            Database::detect_db_type("mysql://user:pass@localhost/db").unwrap(),
            DatabaseType::MySQL
        );
        assert_eq!(
            Database::detect_db_type("mysql://localhost:3306/artemis").unwrap(),
            DatabaseType::MySQL
        );
    }

    #[test]
    fn test_detect_db_type_unsupported() {
        assert!(Database::detect_db_type("postgres://localhost").is_err());
        assert!(Database::detect_db_type("mongodb://localhost").is_err());
        assert!(Database::detect_db_type("invalid").is_err());
        assert!(Database::detect_db_type("").is_err());
    }

    // ========== sanitize_url 测试 ==========

    #[test]
    fn test_sanitize_url_mysql_with_password() {
        assert_eq!(
            Database::sanitize_url("mysql://user:password@localhost/db"),
            "mysql://user:****@localhost/db"
        );
        assert_eq!(
            Database::sanitize_url("mysql://admin:secret123@127.0.0.1:3306/artemis"),
            "mysql://admin:****@127.0.0.1:3306/artemis"
        );
    }

    #[test]
    fn test_sanitize_url_sqlite_no_password() {
        assert_eq!(
            Database::sanitize_url("sqlite://test.db"),
            "sqlite://test.db"
        );
        assert_eq!(
            Database::sanitize_url("sqlite::memory:"),
            "sqlite::memory:"
        );
    }

    #[test]
    fn test_sanitize_url_no_at_symbol() {
        assert_eq!(
            Database::sanitize_url("mysql://localhost/db"),
            "mysql://localhost/db"
        );
    }

    #[test]
    fn test_sanitize_url_empty_password() {
        assert_eq!(
            Database::sanitize_url("mysql://user:@localhost/db"),
            "mysql://user:****@localhost/db"
        );
    }

    // ========== 异步测试 (需要实际数据库) ==========

    #[tokio::test]
    async fn test_new_sqlite_memory() {
        let db = Database::new("sqlite::memory:", 5).await.unwrap();
        assert_eq!(db.db_type(), DatabaseType::SQLite);
    }

    #[tokio::test]
    async fn test_health_check_sqlite() {
        let db = Database::new("sqlite::memory:", 5).await.unwrap();
        assert!(db.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_db_type() {
        let db = Database::new("sqlite::memory:", 5).await.unwrap();
        assert_eq!(db.db_type(), DatabaseType::SQLite);
    }

    #[tokio::test]
    async fn test_conn() {
        let db = Database::new("sqlite::memory:", 5).await.unwrap();
        let _conn = db.conn();
        // 只要不panic就说明获取连接成功
    }

    #[tokio::test]
    async fn test_clone() {
        let db = Database::new("sqlite::memory:", 5).await.unwrap();
        let cloned = db.clone();
        assert_eq!(cloned.db_type(), db.db_type());
    }

    #[tokio::test]
    async fn test_run_migrations() {
        let db = Database::new("sqlite::memory:", 5).await.unwrap();
        // run_migrations 目前只是占位符,应该不会失败
        let result = db.run_migrations().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_close() {
        let db = Database::new("sqlite::memory:", 5).await.unwrap();
        let result = db.close().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_new_invalid_url() {
        let result = Database::new("invalid://url", 5).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_connections() {
        let db = Database::new("sqlite::memory:", 10).await.unwrap();
        // 创建多个数据库实例
        let db2 = Database::new("sqlite::memory:", 10).await.unwrap();

        assert!(db.health_check().await.is_ok());
        assert!(db2.health_check().await.is_ok());
    }
}
