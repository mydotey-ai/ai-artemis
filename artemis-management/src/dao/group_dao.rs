use artemis_core::model::{ServiceGroup, GroupTag, GroupType, GroupStatus};
use sqlx::{Pool, Any, Row};

pub struct GroupDao {
    pool: Pool<Any>,
}

impl GroupDao {
    pub fn new(pool: Pool<Any>) -> Self {
        Self { pool }
    }

    /// 插入服务分组
    pub async fn insert_group(&self, group: &ServiceGroup) -> anyhow::Result<()> {
        let group_type_str = match group.group_type {
            GroupType::Physical => "physical",
            GroupType::Logical => "logical",
        };

        let metadata_json = serde_json::to_string(&group.metadata.as_ref().unwrap_or(&std::collections::HashMap::new()))?;

        sqlx::query(
            r#"
            INSERT INTO service_group (group_id, group_name, group_type, service_id, region_id, zone_id, description, metadata)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&group.name)
        .bind(&group.name)
        .bind(group_type_str)
        .bind(&group.service_id)
        .bind(&group.region_id)
        .bind(&group.zone_id)
        .bind(&group.description)
        .bind(metadata_json)
        .execute(&self.pool)
        .await?;

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

        let metadata_json = serde_json::to_string(&group.metadata.as_ref().unwrap_or(&std::collections::HashMap::new()))?;

        sqlx::query(
            r#"
            UPDATE service_group
            SET group_name = ?, group_type = ?, service_id = ?, region_id = ?, zone_id = ?, description = ?, metadata = ?, updated_at = CURRENT_TIMESTAMP
            WHERE group_id = ?
            "#
        )
        .bind(&group.name)
        .bind(group_type_str)
        .bind(&group.service_id)
        .bind(&group.region_id)
        .bind(&group.zone_id)
        .bind(&group.description)
        .bind(metadata_json)
        .bind(&group.name)
        .execute(&self.pool)
        .await?;

        // 删除旧标签并插入新标签
        sqlx::query("DELETE FROM service_group_tag WHERE group_id = ?")
            .bind(&group.name)
            .execute(&self.pool)
            .await?;

        if let Some(tags) = &group.tags {
            for tag in tags {
                self.insert_tag(&group.name, tag).await?;
            }
        }

        Ok(())
    }

    /// 删除服务分组
    pub async fn delete_group(&self, group_id: &str) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM service_group WHERE group_id = ?")
            .bind(group_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// 获取服务分组
    pub async fn get_group(&self, group_id: &str) -> anyhow::Result<Option<ServiceGroup>> {
        let row = sqlx::query(
            r#"
            SELECT group_id, group_name, group_type, service_id, region_id, zone_id, description, metadata, created_at
            FROM service_group
            WHERE group_id = ?
            "#
        )
        .bind(group_id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                let group_type_str: String = row.get("group_type");
                let group_type = match group_type_str.as_str() {
                    "physical" => GroupType::Physical,
                    "logical" => GroupType::Logical,
                    _ => GroupType::Physical,
                };

                let metadata_json: String = row.get("metadata");
                let metadata = serde_json::from_str(&metadata_json).ok();

                let tags = self.get_tags(group_id).await?;

                let created_at_str: Option<String> = row.get("created_at");
                let created_at = created_at_str.and_then(|s| {
                    chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
                        .ok()
                        .map(|dt| dt.and_utc().timestamp())
                });

                Ok(Some(ServiceGroup {
                    group_id: None,
                    service_id: row.get("service_id"),
                    region_id: row.get("region_id"),
                    zone_id: row.get("zone_id"),
                    name: row.get("group_name"),
                    group_type,
                    status: GroupStatus::Active,
                    description: row.get("description"),
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
        let rows = sqlx::query(
            r#"
            SELECT group_id, group_name, group_type, service_id, region_id, zone_id, description, metadata, created_at
            FROM service_group
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut groups = Vec::new();
        for row in rows {
            let group_id: String = row.get("group_id");
            let group_type_str: String = row.get("group_type");
            let group_type = match group_type_str.as_str() {
                "physical" => GroupType::Physical,
                "logical" => GroupType::Logical,
                _ => GroupType::Physical,
            };

            let metadata_json: String = row.get("metadata");
            let metadata = serde_json::from_str(&metadata_json).ok();

            let tags = self.get_tags(&group_id).await?;

            let created_at_str: Option<String> = row.get("created_at");
            let created_at = created_at_str.and_then(|s| {
                chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S")
                    .ok()
                    .map(|dt| dt.and_utc().timestamp())
            });

            groups.push(ServiceGroup {
                group_id: None,
                service_id: row.get("service_id"),
                region_id: row.get("region_id"),
                zone_id: row.get("zone_id"),
                name: row.get("group_name"),
                group_type,
                status: GroupStatus::Active,
                description: row.get("description"),
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
        sqlx::query(
            r#"
            INSERT INTO service_group_tag (group_id, tag_key, tag_value)
            VALUES (?, ?, ?)
            ON CONFLICT(group_id, tag_key) DO UPDATE SET tag_value = excluded.tag_value
            "#
        )
        .bind(group_id)
        .bind(&tag.key)
        .bind(&tag.value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    /// 获取分组的所有标签
    async fn get_tags(&self, group_id: &str) -> anyhow::Result<Vec<GroupTag>> {
        let rows = sqlx::query(
            r#"
            SELECT tag_key, tag_value
            FROM service_group_tag
            WHERE group_id = ?
            "#
        )
        .bind(group_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .into_iter()
            .map(|row| GroupTag {
                key: row.get("tag_key"),
                value: row.get("tag_value"),
            })
            .collect())
    }
}
