use crate::model::{GroupStatus, GroupTag, GroupType, ServiceGroup};
use sea_orm::sea_query::Value;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

pub struct GroupDao {
    conn: DatabaseConnection,
}

impl GroupDao {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    /// 插入服务分组 (兼容 Java 列名)
    pub async fn insert_group(&self, group: &ServiceGroup) -> anyhow::Result<()> {
        let group_type_str = match group.group_type {
            GroupType::Physical => "physical",
            GroupType::Logical => "logical",
        };

        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            INSERT INTO service_group (NAME, SERVICE_ID, REGION_ID, ZONE_ID, APP_ID, DESCRIPTION, STATUS, type)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            vec![
                Value::from(&group.name),
                Value::from(&group.service_id),
                Value::from(&group.region_id),
                Value::from(&group.zone_id),
                Value::from(&group.service_id), // APP_ID 使用 service_id
                Value::from(group.description.as_deref().unwrap_or("")),
                Value::from("active"),
                Value::from(group_type_str),
            ],
        );

        self.conn.execute(stmt).await?;

        // 插入标签
        if let Some(tags) = &group.tags {
            for tag in tags {
                self.insert_tag(&group.name, tag).await?;
            }
        }

        Ok(())
    }

    /// 更新服务分组 (兼容 Java 列名)
    pub async fn update_group(&self, group: &ServiceGroup) -> anyhow::Result<()> {
        let group_type_str = match group.group_type {
            GroupType::Physical => "physical",
            GroupType::Logical => "logical",
        };

        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            UPDATE service_group
            SET NAME = ?, SERVICE_ID = ?, REGION_ID = ?, ZONE_ID = ?, APP_ID = ?, DESCRIPTION = ?, STATUS = ?, type = ?, DataChange_LastTime = CURRENT_TIMESTAMP
            WHERE NAME = ?
            "#,
            vec![
                Value::from(&group.name),
                Value::from(&group.service_id),
                Value::from(&group.region_id),
                Value::from(&group.zone_id),
                Value::from(&group.service_id), // APP_ID 使用 service_id
                Value::from(group.description.as_deref().unwrap_or("")),
                Value::from("active"),
                Value::from(group_type_str),
                Value::from(&group.name),
            ],
        );

        self.conn.execute(stmt).await?;

        // 删除旧标签并插入新标签
        let delete_stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "DELETE FROM service_group_tag WHERE GROUP_ID = ?",
            vec![Value::from(&group.name)],
        );
        self.conn.execute(delete_stmt).await?;

        if let Some(tags) = &group.tags {
            for tag in tags {
                self.insert_tag(&group.name, tag).await?;
            }
        }

        Ok(())
    }

    /// 删除服务分组 (兼容 Java 列名)
    pub async fn delete_group(&self, group_id: &str) -> anyhow::Result<()> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "DELETE FROM service_group WHERE NAME = ?",
            vec![Value::from(group_id)],
        );
        self.conn.execute(stmt).await?;
        Ok(())
    }

    /// 获取服务分组 (兼容 Java 列名)
    pub async fn get_group(&self, group_id: &str) -> anyhow::Result<Option<ServiceGroup>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            SELECT NAME, SERVICE_ID, REGION_ID, ZONE_ID, DESCRIPTION, type
            FROM service_group
            WHERE NAME = ?
            "#,
            vec![Value::from(group_id)],
        );

        let result = self.conn.query_one(stmt).await?;

        match result {
            Some(row) => {
                let group_type_str: String = row.try_get("", "type")?;
                let group_type = match group_type_str.as_str() {
                    "physical" => GroupType::Physical,
                    "logical" => GroupType::Logical,
                    _ => GroupType::Physical,
                };

                let tags = self.get_tags(group_id).await?;

                Ok(Some(ServiceGroup {
                    group_id: None,
                    service_id: row.try_get("", "SERVICE_ID")?,
                    region_id: row.try_get("", "REGION_ID")?,
                    zone_id: row.try_get("", "ZONE_ID")?,
                    name: row.try_get("", "NAME")?,
                    group_type,
                    status: GroupStatus::Active,
                    description: row.try_get("", "DESCRIPTION")?,
                    tags: if tags.is_empty() { None } else { Some(tags) },
                    metadata: None,
                    created_at: None,
                    updated_at: None,
                }))
            }
            None => Ok(None),
        }
    }

    /// 列出所有服务分组 (兼容 Java 列名)
    pub async fn list_groups(&self) -> anyhow::Result<Vec<ServiceGroup>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            SELECT NAME, SERVICE_ID, REGION_ID, ZONE_ID, DESCRIPTION, type
            FROM service_group
            "#,
            vec![],
        );

        let rows = self.conn.query_all(stmt).await?;

        let mut groups = Vec::new();
        for row in rows {
            let group_type_str: String = row.try_get("", "type")?;
            let group_type = match group_type_str.as_str() {
                "physical" => GroupType::Physical,
                "logical" => GroupType::Logical,
                _ => GroupType::Physical,
            };

            let name: String = row.try_get("", "NAME")?;
            let tags = self.get_tags(&name).await?;

            groups.push(ServiceGroup {
                group_id: None,
                service_id: row.try_get("", "SERVICE_ID")?,
                region_id: row.try_get("", "REGION_ID")?,
                zone_id: row.try_get("", "ZONE_ID")?,
                name,
                group_type,
                status: GroupStatus::Active,
                description: row.try_get("", "DESCRIPTION")?,
                tags: if tags.is_empty() { None } else { Some(tags) },
                metadata: None,
                created_at: None,
                updated_at: None,
            });
        }

        Ok(groups)
    }

    /// 插入标签 (兼容 Java 列名)
    async fn insert_tag(&self, group_id: &str, tag: &GroupTag) -> anyhow::Result<()> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            INSERT INTO service_group_tag (GROUP_ID, TAG, VALUE)
            VALUES (?, ?, ?)
            "#,
            vec![Value::from(group_id), Value::from(&tag.key), Value::from(&tag.value)],
        );
        self.conn.execute(stmt).await?;
        Ok(())
    }

    /// 获取分组的所有标签 (兼容 Java 列名)
    async fn get_tags(&self, group_id: &str) -> anyhow::Result<Vec<GroupTag>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            SELECT TAG, VALUE
            FROM service_group_tag
            WHERE GROUP_ID = ?
            "#,
            vec![Value::from(group_id)],
        );

        let rows = self.conn.query_all(stmt).await?;

        Ok(rows
            .into_iter()
            .map(|row| GroupTag {
                key: row.try_get("", "TAG").unwrap_or_default(),
                value: row.try_get("", "VALUE").unwrap_or_default(),
            })
            .collect())
    }
}
