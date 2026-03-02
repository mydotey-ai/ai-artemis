# Phase 17: 金丝雀发布

**优先级**: P0 (必须完成 - Java版本有此功能)
**状态**: ✅ **已完成** (2026-02-14)
**实际时间**: 1-2天
**依赖**: Phase 13 (分组路由)
**目标**: 100%对齐Java版本,实现基于IP白名单的完整金丝雀发布功能

---

## 📋 目标

实现基于 IP 白名单的金丝雀发布功能,支持特定 IP 范围访问灰度实例,实现精细化的流量控制。

### 核心功能

1. **金丝雀 IP 白名单管理** - 配置可访问灰度实例的 IP
2. **基于 IP 的流量路由** - 白名单 IP 路由到灰度实例
3. **金丝雀配置 API** - 配置管理接口
4. **与分组路由集成** - 结合分组实现灰度发布
5. **IP 范围支持** - 支持 CIDR 格式

---

## 🎯 任务清单

### Task 1: 金丝雀配置数据模型

**文件**: `artemis-common/src/model/canary.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryConfig {
    pub service_id: String,
    pub ip_whitelist: Vec<String>, // CIDR 格式: "192.168.1.0/24"
    pub canary_group_id: String,   // 金丝雀分组 ID
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryRule {
    pub service_id: String,
    pub ip_ranges: Vec<IpRange>,
    pub target_group_id: String,
}

pub struct IpRange {
    pub start: IpAddr,
    pub end: IpAddr,
}
```

---

### Task 2: CanaryManager 实现

**文件**: `artemis-management/src/canary.rs`

```rust
pub struct CanaryManager {
    canary_configs: Arc<DashMap<String, CanaryConfig>>,
}

impl CanaryManager {
    pub fn update_canary_ips(&self, config: CanaryConfig) -> Result<()>;
    pub fn get_canary_config(&self, service_id: &str) -> Option<CanaryConfig>;
    pub fn remove_canary_config(&self, service_id: &str) -> bool;
    pub fn is_canary_client(&self, client_ip: &str, service_id: &str) -> bool;
}

// IP 匹配逻辑
impl CanaryManager {
    fn match_ip(&self, client_ip: &str, whitelist: &[String]) -> bool {
        let client_ip: IpAddr = client_ip.parse().unwrap();

        for cidr in whitelist {
            if self.ip_in_range(&client_ip, cidr) {
                return true;
            }
        }

        false
    }

    fn ip_in_range(&self, ip: &IpAddr, cidr: &str) -> bool {
        // 使用 ipnetwork crate
        let network: IpNetwork = cidr.parse().unwrap();
        network.contains(*ip)
    }
}
```

---

### Task 3: 金丝雀 API 实现 (1个端点)

**文件**: `artemis-server/src/api/management/canary.rs`

#### API 端点

1. **更新金丝雀 IP 白名单**
   ```
   POST /api/management/canary/update-canary-ips.json

   Request:
   {
     "service_id": "my-service",
     "ip_whitelist": [
       "192.168.1.0/24",
       "10.0.0.100/32",
       "172.16.0.0/16"
     ],
     "canary_group_id": "group-canary",
     "enabled": true
   }

   Response:
   {
     "response_status": {
       "status": "success"
     }
   }
   ```

---

## 📊 实施成果预期

| 组件 | 预计代码行数 |
|------|-------------|
| 数据模型 | ~100行 |
| CanaryManager | ~150行 |
| Canary API | ~80行 |
| CanaryDiscoveryFilter | ~100行 |
| **总计** | **~430行** |

---

## 💡 路由逻辑

### 与分组路由集成

**CanaryDiscoveryFilter**:
```rust
impl DiscoveryFilter for CanaryDiscoveryFilter {
    fn filter(&self, instances: Vec<Instance>, context: &RouteContext) -> Vec<Instance> {
        let client_ip = context.client_ip.as_ref();

        if let Some(config) = self.canary_manager.get_canary_config(&context.service_id) {
            if config.enabled && self.is_canary_client(client_ip, &config.ip_whitelist) {
                // 白名单 IP: 返回金丝雀分组实例
                return self.filter_by_group(&instances, &config.canary_group_id);
            }
        }

        // 非白名单 IP: 返回正常实例
        instances
    }
}
```

---

## 📝 使用场景

### 场景 1: 内部员工灰度

```bash
# 1. 部署金丝雀版本到 group-canary
POST /api/management/group/insert-group-instances.json
{
  "group_id": "group-canary",
  "instances": [...]  # 新版本实例
}

# 2. 配置员工 IP 白名单
POST /api/management/canary/update-canary-ips.json
{
  "service_id": "my-service",
  "ip_whitelist": ["10.0.0.0/8"],  # 公司内网
  "canary_group_id": "group-canary",
  "enabled": true
}

# 3. 内网员工访问 my-service → 金丝雀版本
# 4. 外网用户访问 my-service → 稳定版本

# 5. 验证无问题后,全量发布
POST /api/management/canary/update-canary-ips.json
{
  "service_id": "my-service",
  "enabled": false  # 关闭金丝雀
}
```

### 场景 2: 特定客户灰度

```bash
# 为 VIP 客户提供新功能

POST /api/management/canary/update-canary-ips.json
{
  "service_id": "premium-feature",
  "ip_whitelist": [
    "203.0.113.10/32",  # VIP 客户 A
    "198.51.100.20/32"   # VIP 客户 B
  ],
  "canary_group_id": "group-premium",
  "enabled": true
}
```

---

## 🧪 测试计划

### 单元测试

1. IP 匹配逻辑 (CIDR)
2. 金丝雀配置 CRUD
3. CanaryDiscoveryFilter

### 集成测试

```bash
# 1. 配置金丝雀 IP 白名单
# 2. 白名单 IP 请求 → 验证路由到金丝雀实例
# 3. 非白名单 IP 请求 → 验证路由到正常实例
# 4. 禁用金丝雀 → 验证所有请求路由到正常实例
```

---

## 📚 依赖库

```toml
[dependencies]
ipnetwork = "0.20"  # CIDR 解析和 IP 范围匹配
```

---

**参考文档**: Java 版本 CanaryService 实现
