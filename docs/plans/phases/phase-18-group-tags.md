# Phase 18: 分组标签管理

**优先级**: P3 (可选)
**状态**: ⚠️ **待完成**
**预计时间**: 1-2天
**依赖**: Phase 13 (分组路由)

---

## 📋 目标

实现服务分组的标签(Tag)管理功能,支持为分组添加元数据标签,增强分组的可管理性和可查询性。

### 核心功能

1. **分组标签 CRUD** - 创建/查询/更新/删除标签
2. **基于标签过滤** - 按标签查询分组
3. **标签继承** - 实例继承分组标签
4. **标签搜索** - 支持标签组合查询
5. **标签验证** - 标签键值格式验证

---

## 🎯 任务清单

### Task 1: 标签数据模型

**文件**: `artemis-core/src/model/group.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupTag {
    pub group_id: String,
    pub tag_key: String,
    pub tag_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagFilter {
    pub tags: HashMap<String, String>,  // key -> value
    pub match_all: bool,  // true: AND, false: OR
}

// 预定义标签键
pub mod well_known_tags {
    pub const ENVIRONMENT: &str = "env";       // prod, staging, dev
    pub const VERSION: &str = "version";       // v1.0, v2.0
    pub const OWNER: &str = "owner";           // team name
    pub const REGION: &str = "region";         // us-east, cn-north
    pub const TIER: &str = "tier";             // frontend, backend, db
}
```

---

### Task 2: TagManager 实现

**文件**: `artemis-management/src/tag.rs`

```rust
pub struct TagManager {
    group_tags: Arc<DashMap<String, HashMap<String, String>>>, // group_id -> tags
}

impl TagManager {
    // 标签 CRUD
    pub fn add_tag(&self, group_id: String, key: String, value: String) -> Result<()>;
    pub fn remove_tag(&self, group_id: &str, key: &str) -> Result<()>;
    pub fn update_tag(&self, group_id: String, key: String, value: String) -> Result<()>;
    pub fn get_tags(&self, group_id: &str) -> Option<HashMap<String, String>>;

    // 批量操作
    pub fn add_tags_batch(&self, group_id: String, tags: HashMap<String, String>) -> Result<()>;
    pub fn remove_tags_batch(&self, group_id: &str, keys: Vec<String>) -> Result<()>;

    // 查询
    pub fn find_groups_by_tag(&self, key: &str, value: &str) -> Vec<String>;
    pub fn find_groups_by_tags(&self, filter: &TagFilter) -> Vec<String>;
}

// 标签验证
impl TagManager {
    fn validate_tag_key(&self, key: &str) -> Result<()> {
        // 1. 长度限制: 1-64 字符
        if key.is_empty() || key.len() > 64 {
            return Err(ArtemisError::InvalidTagKey);
        }

        // 2. 格式限制: 字母数字和 .-_
        if !key.chars().all(|c| c.is_alphanumeric() || c == '.' || c == '-' || c == '_') {
            return Err(ArtemisError::InvalidTagKey);
        }

        Ok(())
    }

    fn validate_tag_value(&self, value: &str) -> Result<()> {
        // 1. 长度限制: 0-256 字符
        if value.len() > 256 {
            return Err(ArtemisError::InvalidTagValue);
        }

        Ok(())
    }
}
```

---

### Task 3: 分组标签 API (5个端点)

**文件**: `artemis-web/src/api/management/group_tags.rs`

#### API 端点

1. **添加分组标签**
   ```
   POST /api/management/group/insert-group-tags.json

   Request:
   {
     "group_id": "group-1",
     "tags": {
       "env": "prod",
       "version": "v2.0",
       "owner": "team-a"
     }
   }
   ```

2. **更新分组标签**
   ```
   POST /api/management/group/update-group-tags.json

   Request:
   {
     "group_id": "group-1",
     "tags": {
       "env": "staging",  # 更新现有标签
       "tier": "backend"  # 添加新标签
     }
   }
   ```

3. **删除分组标签**
   ```
   POST /api/management/group/delete-group-tags.json

   Request:
   {
     "group_id": "group-1",
     "tag_keys": ["version", "tier"]
   }
   ```

4. **查询分组标签**
   ```
   POST /api/management/group/get-group-tags.json

   Request:
   {
     "group_id": "group-1"
   }

   Response:
   {
     "group_tags": [
       {"tag_key": "env", "tag_value": "prod"},
       {"tag_key": "owner", "tag_value": "team-a"}
     ]
   }
   ```

5. **查询所有分组标签**
   ```
   POST /api/management/group/get-all-group-tags.json

   Response:
   {
     "group_tags": {
       "group-1": {"env": "prod", "owner": "team-a"},
       "group-2": {"env": "staging", "tier": "frontend"}
     }
   }
   ```

---

## 📊 实施成果预期

| 组件 | 预计代码行数 |
|------|-------------|
| 数据模型 | ~100行 |
| TagManager | ~200行 |
| 标签 API | ~250行 |
| **总计** | **~550行** |

---

## 💡 使用场景

### 场景 1: 环境标签管理

```bash
# 1. 为生产环境分组添加标签
POST /api/management/group/insert-group-tags.json
{
  "group_id": "group-prod",
  "tags": {
    "env": "prod",
    "region": "us-east",
    "tier": "backend"
  }
}

# 2. 查询所有生产环境分组
POST /api/management/group/find-groups-by-tags.json
{
  "filter": {
    "tags": {"env": "prod"},
    "match_all": false
  }
}

# 返回: ["group-prod", "group-prod-2", ...]
```

### 场景 2: 版本管理

```bash
# 标记分组版本
POST /api/management/group/insert-group-tags.json
{
  "group_id": "group-v2",
  "tags": {
    "version": "v2.0",
    "release_date": "2026-02-14"
  }
}

# 查询特定版本的分组
POST /api/management/group/find-groups-by-tags.json
{
  "filter": {
    "tags": {"version": "v2.0"}
  }
}
```

### 场景 3: 团队归属

```bash
# 标记分组所有者
POST /api/management/group/insert-group-tags.json
{
  "group_id": "group-team-a",
  "tags": {
    "owner": "team-a",
    "contact": "team-a@example.com"
  }
}

# 查询某个团队的所有分组
POST /api/management/group/find-groups-by-tags.json
{
  "filter": {
    "tags": {"owner": "team-a"}
  }
}
```

---

## 🔍 高级查询

### 组合标签查询

```bash
# AND 查询: 同时满足多个标签
POST /api/management/group/find-groups-by-tags.json
{
  "filter": {
    "tags": {
      "env": "prod",
      "region": "us-east",
      "tier": "backend"
    },
    "match_all": true  # AND
  }
}

# OR 查询: 满足任一标签
POST /api/management/group/find-groups-by-tags.json
{
  "filter": {
    "tags": {
      "env": "prod",
      "env": "staging"
    },
    "match_all": false  # OR
  }
}
```

---

## 🔗 与分组路由集成

### 基于标签的路由规则

```rust
// 示例: 根据环境标签路由
pub struct TagBasedRoutingFilter {
    tag_manager: Arc<TagManager>,
}

impl DiscoveryFilter for TagBasedRoutingFilter {
    fn filter(&self, instances: Vec<Instance>, context: &RouteContext) -> Vec<Instance> {
        // 从请求头获取环境标签
        let env = context.headers.get("X-Env");

        if let Some(env) = env {
            // 查找匹配环境的分组
            let group_ids = self.tag_manager.find_groups_by_tag("env", env);

            // 过滤出属于这些分组的实例
            return instances.into_iter()
                .filter(|inst| group_ids.contains(&inst.group_id))
                .collect();
        }

        instances
    }
}
```

---

## 🧪 测试计划

### 单元测试

1. 标签 CRUD 操作
2. 标签验证逻辑
3. 标签查询逻辑

### 集成测试

```bash
# 1. 创建分组并添加标签
# 2. 查询标签验证
# 3. 更新标签
# 4. 基于标签查询分组
# 5. 删除标签
# 6. 组合标签查询 (AND/OR)
```

---

## 📝 标签最佳实践

### 标签命名规范

1. **使用小写** - `env` 而非 `ENV`
2. **使用连字符** - `release-date` 而非 `releaseDate`
3. **语义化** - 使用有意义的名称
4. **预定义常用标签** - 在 well_known_tags 中定义

### 标签使用建议

1. **环境标签**: `env: prod/staging/dev`
2. **版本标签**: `version: v1.0/v2.0`
3. **区域标签**: `region: us-east/cn-north`
4. **团队标签**: `owner: team-a/team-b`
5. **层级标签**: `tier: frontend/backend/db`

---

**参考文档**: Java 版本 GroupTagService 实现
