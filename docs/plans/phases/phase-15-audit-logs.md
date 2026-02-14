# Phase 15: 操作审计日志

**优先级**: P3 (可选)
**状态**: ⚠️ **待完成**
**预计时间**: 2-3天
**依赖**: Phase 14 (数据持久化)

---

## 📋 目标

实现完整的操作审计日志功能,记录所有管理操作的历史,支持操作回溯和审计合规。

### 核心功能

1. **操作日志记录** - 记录所有管理操作
2. **日志查询 API** - 9个查询端点
3. **操作历史回溯** - 按时间/操作人/操作类型查询
4. **日志归档** - 定期归档历史日志
5. **日志导出** - 支持导出为 CSV/JSON

---

## 🎯 任务清单

### Task 1: 操作日志数据模型

**文件**: `artemis-core/src/model/audit.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationLog {
    pub log_id: String,
    pub operation_type: OperationType,
    pub resource_type: ResourceType,
    pub resource_id: String,
    pub operator_id: String,
    pub operation_time: u64,
    pub operation_detail: String, // JSON
    pub result: OperationResult,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    Create,
    Update,
    Delete,
    PullIn,
    PullOut,
    Release,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResourceType {
    Instance,
    Server,
    ServiceGroup,
    RouteRule,
    Zone,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationResult {
    Success,
    Failure,
}
```

---

### Task 2: 日志记录器实现

**文件**: `artemis-management/src/audit/logger.rs`

```rust
pub struct AuditLogger {
    dao: Arc<OperationLogDao>,
}

impl AuditLogger {
    pub async fn log_operation(&self, log: OperationLog) -> Result<()> {
        self.dao.insert_log(&log).await?;
        Ok(())
    }

    pub async fn log_instance_operation(&self, ...) -> Result<()>;
    pub async fn log_server_operation(&self, ...) -> Result<()>;
    pub async fn log_group_operation(&self, ...) -> Result<()>;
    pub async fn log_route_operation(&self, ...) -> Result<()>;
}
```

---

### Task 3: 日志查询 API (9个端点)

#### 实例操作日志查询 (3个)

1. **查询实例操作日志**
   ```
   POST /api/management/audit/instance-logs.json
   Request: { "instance_key": {...}, "start_time": 0, "end_time": 0 }
   ```

2. **查询实例操作日志(分页)**
   ```
   POST /api/management/audit/instance-logs-paged.json
   Request: { "filter": {...}, "page": 1, "page_size": 20 }
   ```

3. **导出实例操作日志**
   ```
   POST /api/management/audit/instance-logs-export.json
   Response: CSV 或 JSON 文件
   ```

#### 服务器操作日志查询 (3个)

4. **查询服务器操作日志**
5. **查询服务器操作日志(分页)**
6. **导出服务器操作日志**

#### 通用日志查询 (3个)

7. **按操作人查询**
   ```
   POST /api/management/audit/logs-by-operator.json
   Request: { "operator_id": "admin", "start_time": 0, "end_time": 0 }
   ```

8. **按时间范围查询**
   ```
   POST /api/management/audit/logs-by-time.json
   ```

9. **按操作类型查询**
   ```
   POST /api/management/audit/logs-by-type.json
   Request: { "operation_type": "pullout", "resource_type": "instance" }
   ```

---

## 📊 实施成果预期

| 组件 | 预计代码行数 |
|------|-------------|
| 数据模型 | ~150行 |
| AuditLogger | ~200行 |
| OperationLogDao | ~150行 |
| 查询 API | ~300行 |
| **总计** | **~800行** |

---

## 📝 业务价值

- ✅ 操作可追溯 - 所有操作都有记录
- ✅ 审计合规 - 满足审计要求
- ✅ 故障排查 - 快速定位问题操作
- ✅ 统计分析 - 操作频率和趋势分析

---

**参考文档**: Java 版本操作日志实现
