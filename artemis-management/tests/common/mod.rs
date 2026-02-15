//! 数据库测试工具
//!
//! 提供数据库测试的通用功能:
//! - 内存数据库创建
//! - 测试数据准备
//! - 事务测试支持

use sea_orm::{Database, DatabaseConnection, DbErr};

/// 创建内存 SQLite 测试数据库
pub async fn create_test_db() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect("sqlite::memory:").await?;

    // 运行迁移脚本创建表结构
    initialize_schema(&db).await?;

    Ok(db)
}

/// 初始化测试数据库 Schema
async fn initialize_schema(db: &DatabaseConnection) -> Result<(), DbErr> {
    use sea_orm::ConnectionTrait;

    // 创建 service_groups 表
    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS service_groups (
            group_id TEXT NOT NULL,
            region_id TEXT NOT NULL,
            description TEXT,
            tags TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY (group_id, region_id)
        )
        "#,
    )
    .await?;

    // 创建 route_rules 表
    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS route_rules (
            rule_id TEXT PRIMARY KEY NOT NULL,
            strategy TEXT NOT NULL,
            unfiltered_group_ids TEXT,
            route_groups TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )
        "#,
    )
    .await?;

    // 创建 route_rule_groups 表
    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS route_rule_groups (
            rule_group_id TEXT PRIMARY KEY NOT NULL,
            app_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )
        "#,
    )
    .await?;

    // 创建 route_rule_mappings 表
    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS route_rule_mappings (
            rule_group_id TEXT NOT NULL,
            rule_id TEXT NOT NULL,
            priority INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY (rule_group_id, rule_id)
        )
        "#,
    )
    .await?;

    // 创建 zone_operations 表
    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS zone_operations (
            operation_id TEXT PRIMARY KEY NOT NULL,
            region_id TEXT NOT NULL,
            zone_id TEXT NOT NULL,
            operation_type TEXT NOT NULL,
            token TEXT NOT NULL,
            operator_id TEXT NOT NULL,
            created_at INTEGER NOT NULL
        )
        "#,
    )
    .await?;

    // 创建 canary_configs 表
    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS canary_configs (
            config_id TEXT PRIMARY KEY NOT NULL,
            region_id TEXT NOT NULL,
            zone_id TEXT,
            service_id TEXT NOT NULL,
            whitelist_ips TEXT NOT NULL,
            enabled INTEGER NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        )
        "#,
    )
    .await?;

    // 创建 group_instances 表
    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS group_instances (
            group_id TEXT NOT NULL,
            region_id TEXT NOT NULL,
            instance_id TEXT NOT NULL,
            zone_id TEXT NOT NULL,
            service_id TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            PRIMARY KEY (group_id, region_id, instance_id)
        )
        "#,
    )
    .await?;

    // 创建 audit_logs 表
    db.execute_unprepared(
        r#"
        CREATE TABLE IF NOT EXISTS audit_logs (
            log_id INTEGER PRIMARY KEY AUTOINCREMENT,
            entity_type TEXT NOT NULL,
            entity_id TEXT NOT NULL,
            action TEXT NOT NULL,
            operator TEXT NOT NULL,
            details TEXT,
            created_at INTEGER NOT NULL
        )
        "#,
    )
    .await?;

    Ok(())
}

/// 清空所有测试表数据
pub async fn clear_test_data(db: &DatabaseConnection) -> Result<(), DbErr> {
    use sea_orm::ConnectionTrait;

    db.execute_unprepared("DELETE FROM service_groups").await?;
    db.execute_unprepared("DELETE FROM route_rules").await?;
    db.execute_unprepared("DELETE FROM route_rule_groups").await?;
    db.execute_unprepared("DELETE FROM route_rule_mappings").await?;
    db.execute_unprepared("DELETE FROM zone_operations").await?;
    db.execute_unprepared("DELETE FROM canary_configs").await?;
    db.execute_unprepared("DELETE FROM group_instances").await?;
    db.execute_unprepared("DELETE FROM audit_logs").await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_test_db() {
        let db = create_test_db().await;
        assert!(db.is_ok());
    }

    #[tokio::test]
    async fn test_clear_test_data() {
        let db = create_test_db().await.unwrap();
        let result = clear_test_data(&db).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_schema_initialization() {
        use sea_orm::ConnectionTrait;

        let db = create_test_db().await.unwrap();

        // 验证表是否创建成功
        let result = db
            .execute_unprepared("SELECT COUNT(*) FROM service_groups")
            .await;
        assert!(result.is_ok());

        let result = db
            .execute_unprepared("SELECT COUNT(*) FROM route_rules")
            .await;
        assert!(result.is_ok());

        let result = db
            .execute_unprepared("SELECT COUNT(*) FROM audit_logs")
            .await;
        assert!(result.is_ok());
    }
}
