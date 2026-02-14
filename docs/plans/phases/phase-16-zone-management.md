# Phase 16: Zone 管理功能

**优先级**: P3 (可选)
**状态**: ⚠️ **待完成**
**预计时间**: 2-3天
**依赖**: Phase 12 (实例管理)

---

## 📋 目标

实现可用区(Zone)级别的流量管理功能,支持批量控制整个可用区的流量接入,提供更粗粒度的流量管理能力。

### 核心功能

1. **Zone 级别拉入/拉出** - 批量控制可用区流量
2. **Zone 状态查询** - 查询可用区操作状态
3. **Zone 操作历史** - 记录操作历史
4. **自动影响实例** - Zone 操作自动影响区内实例
5. **优先级管理** - Zone > Server > Instance

---

## 🎯 任务清单

### Task 1: Zone 操作数据模型

**文件**: `artemis-core/src/model/management.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneOperation {
    pub zone_id: String,
    pub region_id: String,
    pub operation: InstanceOperation, // PullIn/PullOut
    pub operator_id: String,
    pub operation_time: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZoneOperationRecord {
    pub zone_id: String,
    pub region_id: String,
    pub operation: InstanceOperation,
    pub operator_id: String,
    pub operation_time: u64,
}
```

---

### Task 2: ZoneManager 实现

**文件**: `artemis-management/src/zone.rs`

```rust
pub struct ZoneManager {
    zone_operations: Arc<DashMap<String, ZoneOperation>>,
}

impl ZoneManager {
    pub fn pull_out_zone(&self, zone_id: String, region_id: String, operator_id: String);
    pub fn pull_in_zone(&self, zone_id: String, region_id: String, operator_id: String);
    pub fn is_zone_down(&self, zone_id: &str, region_id: &str) -> bool;
    pub fn get_zone_operations(&self, filter: ZoneOperationFilter) -> Vec<ZoneOperation>;
}
```

---

### Task 3: Zone API 实现 (5个端点)

**文件**: `artemis-web/src/api/management/zone.rs`

#### API 端点

1. **拉出/拉入 Zone**
   ```
   POST /api/management/zone/operate-zone-operations.json

   Request:
   {
     "zone_ids": ["zone-1", "zone-2"],
     "region_id": "us-east",
     "operation": "pullout" | "pullin",
     "operator_id": "admin"
   }
   ```

2. **查询 Zone 状态**
   ```
   POST /api/management/zone/is-zone-down.json

   Request:
   {
     "zone_id": "zone-1",
     "region_id": "us-east"
   }

   Response:
   {
     "is_down": true
   }
   ```

3. **查询 Zone 操作记录**
   ```
   POST /api/management/zone/get-zone-operations.json

   Request:
   {
     "zone_ids": ["zone-1"],
     "region_id": "us-east"
   }

   Response:
   {
     "zone_operations": [...]
   }
   ```

4. **查询所有 Zone 操作**
   ```
   POST /api/management/zone/get-all-zone-operations.json

   Response:
   {
     "zone_operations": [...]
   }
   ```

5. **批量操作 Zones**
   ```
   POST /api/management/zone/operate-zones-batch.json
   ```

---

## 📊 实施成果预期

| 组件 | 预计代码行数 |
|------|-------------|
| 数据模型 | ~80行 |
| ZoneManager | ~200行 |
| Zone API | ~200行 |
| **总计** | **~480行** |

---

## 💡 优先级规则

```rust
pub fn is_instance_down(&self, key: &InstanceKey) -> bool {
    // 1. Zone 优先级最高
    if self.zone_manager.is_zone_down(&key.zone_id, &key.region_id) {
        return true;
    }

    // 2. Server 级别次之
    if self.instance_manager.is_server_down(&key.ip, &key.region_id) {
        return true;
    }

    // 3. Instance 级别最低
    self.instance_manager.is_instance_down(key)
}
```

---

## 📝 使用场景

### 场景: 可用区维护

```bash
# 1. 拉出整个可用区
curl -X POST .../zone/operate-zone-operations.json \
  -d '{"zone_ids": ["zone-1"], "operation": "pullout"}'

# 2. zone-1 中所有实例自动停止流量

# 3. 执行可用区维护

# 4. 恢复可用区流量
curl -X POST .../zone/operate-zone-operations.json \
  -d '{"zone_ids": ["zone-1"], "operation": "pullin"}'
```

---

**参考文档**: Java 版本 ZoneService 实现
