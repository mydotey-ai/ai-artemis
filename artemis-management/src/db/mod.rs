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
    /// 注意: SeaORM使用自己的migration系统
    /// 需要将现有SQL migrations转换为SeaORM migrations
    pub async fn run_migrations(&self) -> anyhow::Result<()> {
        tracing::info!("Running database migrations for {:?}", self.db_type);

        // TODO: 实现SeaORM migrations
        // 临时方案: 使用raw SQL执行现有migrations
        tracing::warn!("Migration system needs to be migrated to SeaORM");

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
