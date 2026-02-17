use artemis_core::model::{GroupStatus, GroupTag, GroupType, ServiceGroup};
use sea_orm::sea_query::Value;
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};

pub struct GroupDao {
    conn: DatabaseConnection,
}

impl GroupDao {
    pub fn new(conn: DatabaseConnection) -> Self {
        Self { conn }
    }

    /// 插入服务分组
    pub async fn insert_group(&self, group: &ServiceGroup) -> anyhow::Result<()> {
        let group_type_str = match group.group_type {
            GroupType::Physical => "physical",
            GroupType::Logical => "logical",
        };

        let metadata_json = serde_json::to_string(
            &group.metadata.as_ref().unwrap_or(&std::collections::HashMap::new()),
        )?;

        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            INSERT INTO service_group (group_id, group_name, group_type, service_id, region_id, zone_id, description, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            vec![
                Value::from(&group.name),
                Value::from(&group.name),
                Value::from(group_type_str),
                Value::from(&group.service_id),
                Value::from(&group.region_id),
                Value::from(&group.zone_id),
                Value::from(group.description.as_deref().unwrap_or("")),
                Value::from(metadata_json),
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

    /// 更新服务分组
    pub async fn update_group(&self, group: &ServiceGroup) -> anyhow::Result<()> {
        let group_type_str = match group.group_type {
            GroupType::Physical => "physical",
            GroupType::Logical => "logical",
        };

        let metadata_json = serde_json::to_string(
            &group.metadata.as_ref().unwrap_or(&std::collections::HashMap::new()),
        )?;

        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            UPDATE service_group
            SET group_name = ?, group_type = ?, service_id = ?, region_id = ?, zone_id = ?, description = ?, metadata = ?, updated_at = CURRENT_TIMESTAMP
            WHERE group_id = ?
            "#,
            vec![
                Value::from(&group.name),
                Value::from(group_type_str),
                Value::from(&group.service_id),
                Value::from(&group.region_id),
                Value::from(&group.zone_id),
                Value::from(group.description.as_deref().unwrap_or("")),
                Value::from(metadata_json),
                Value::from(&group.name),
            ],
        );

        self.conn.execute(stmt).await?;

        // 删除旧标签并插入新标签
        let delete_stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "DELETE FROM service_group_tag WHERE group_id = ?",
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

    /// 删除服务分组
    pub async fn delete_group(&self, group_id: &str) -> anyhow::Result<()> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            "DELETE FROM service_group WHERE group_id = ?",
            vec![Value::from(group_id)],
        );
        self.conn.execute(stmt).await?;
        Ok(())
    }

    /// 获取服务分组
    pub async fn get_group(&self, group_id: &str) -> anyhow::Result<Option<ServiceGroup>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            SELECT group_id, group_name, group_type, service_id, region_id, zone_id, description, metadata, created_at
            FROM service_group
            WHERE group_id = ?
            "#,
            vec![Value::from(group_id)],
        );

        let result = self.conn.query_one(stmt).await?;

        match result {
            Some(row) => {
                let group_type_str: String = row.try_get("", "group_type")?;
                let group_type = match group_type_str.as_str() {
                    "physical" => GroupType::Physical,
                    "logical" => GroupType::Logical,
                    _ => GroupType::Physical,
                };

                let metadata_json: String = row.try_get("", "metadata")?;
                let metadata = serde_json::from_str(&metadata_json).ok();

                let tags = self.get_tags(group_id).await?;

                let created_at_str: Option<String> = row.try_get("", "created_at").ok();
                let created_at = created_at_str.and_then(|s| {
                    chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
                        .ok()
                        .map(|dt| dt.and_utc().timestamp())
                });

                Ok(Some(ServiceGroup {
                    group_id: None,
                    service_id: row.try_get("", "service_id")?,
                    region_id: row.try_get("", "region_id")?,
                    zone_id: row.try_get("", "zone_id")?,
                    name: row.try_get("", "group_name")?,
                    group_type,
                    status: GroupStatus::Active,
                    description: row.try_get("", "description")?,
                    tags: if tags.is_empty() { None } else { Some(tags) },
                    metadata,
                    created_at,
                    updated_at: None,
                }))
            }
            None => Ok(None),
        }
    }

    /// 列出所有服务分组
    pub async fn list_groups(&self) -> anyhow::Result<Vec<ServiceGroup>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            SELECT group_id, group_name, group_type, service_id, region_id, zone_id, description, metadata, created_at
            FROM service_group
            "#,
            vec![],
        );

        let rows = self.conn.query_all(stmt).await?;

        let mut groups = Vec::new();
        for row in rows {
            let group_id: String = row.try_get("", "group_id")?;
            let group_type_str: String = row.try_get("", "group_type")?;
            let group_type = match group_type_str.as_str() {
                "physical" => GroupType::Physical,
                "logical" => GroupType::Logical,
                _ => GroupType::Physical,
            };

            let metadata_json: String = row.try_get("", "metadata")?;
            let metadata = serde_json::from_str(&metadata_json).ok();

            let tags = self.get_tags(&group_id).await?;

            let created_at_str: Option<String> = row.try_get("", "created_at").ok();
            let created_at = created_at_str.and_then(|s| {
                chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
                    .ok()
                    .map(|dt| dt.and_utc().timestamp())
            });

            groups.push(ServiceGroup {
                group_id: None,
                service_id: row.try_get("", "service_id")?,
                region_id: row.try_get("", "region_id")?,
                zone_id: row.try_get("", "zone_id")?,
                name: row.try_get("", "group_name")?,
                group_type,
                status: GroupStatus::Active,
                description: row.try_get("", "description")?,
                tags: if tags.is_empty() { None } else { Some(tags) },
                metadata,
                created_at,
                updated_at: None,
            });
        }

        Ok(groups)
    }

    /// 插入标签
    async fn insert_tag(&self, group_id: &str, tag: &GroupTag) -> anyhow::Result<()> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            INSERT INTO service_group_tag (group_id, tag_key, tag_value)
            VALUES (?, ?, ?)
            ON CONFLICT(group_id, tag_key) DO UPDATE SET tag_value = excluded.tag_value
            "#,
            vec![Value::from(group_id), Value::from(&tag.key), Value::from(&tag.value)],
        );
        self.conn.execute(stmt).await?;
        Ok(())
    }

    /// 获取分组的所有标签
    async fn get_tags(&self, group_id: &str) -> anyhow::Result<Vec<GroupTag>> {
        let stmt = Statement::from_sql_and_values(
            self.conn.get_database_backend(),
            r#"
            SELECT tag_key, tag_value
            FROM service_group_tag
            WHERE group_id = ?
            "#,
            vec![Value::from(group_id)],
        );

        let rows = self.conn.query_all(stmt).await?;

        Ok(rows
            .into_iter()
            .map(|row| GroupTag {
                key: row.try_get("", "tag_key").unwrap_or_default(),
                value: row.try_get("", "tag_value").unwrap_or_default(),
            })
            .collect())
    }
}
