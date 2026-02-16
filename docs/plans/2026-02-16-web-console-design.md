# Artemis Web 控制台 - 架构设计文档

**文档状态**: ✅ 最新
**版本**: 1.0.0
**创建日期**: 2026-02-16
**最后更新**: 2026-02-17
**设计状态**: 已完成
**实施状态**: ✅ 已实现（2 天完成）
**实施方式**: 全功能并行开发（原计划 6 周，实际 2 天）
**相关文档**: [项目完成总结](../web-console/project-summary.md)

---

## 目录

1. [项目概述](#1-项目概述)
2. [技术架构](#2-技术架构)
3. [功能模块设计](#3-功能模块设计)
4. [数据流和实时通信](#4-数据流和实时通信)
5. [后端 API 对接](#5-后端-api-对接)
6. [UI/UX 设计](#6-uiux-设计)
7. [实施计划](#7-实施计划)
8. [风险评估](#8-风险评估)
9. [成功标准](#9-成功标准)

---

## 1. 项目概述

### 1.1 项目背景

Artemis 是一个高性能的微服务注册中心（Rust 实现），已经提供了完整的 REST API 和 CLI 工具。为了提升用户体验和运维效率，需要开发一个基于浏览器的 Web 控制台，提供可视化的管理界面。

### 1.2 项目目标

- **可视化管理**：通过图形化界面管理服务实例、集群节点、路由规则
- **实时监控**：展示集群状态、服务健康度、QPS 趋势
- **操作审计**：记录并查询所有管理操作的历史
- **企业级功能**：支持用户权限管理、金丝雀发布、Zone 批量操作

### 1.3 核心特性

- ⚡ **现代化技术栈**：React 18 + TypeScript + Material-UI
- 🎨 **优秀的用户体验**：Material Design 规范，深色/浅色主题
- 🔄 **实时更新**：WebSocket 推送服务变更
- 🌐 **国际化支持**：面向全球用户，英文优先
- 📱 **响应式设计**：支持桌面端和平板端
- 🔐 **权限控制**：基于角色的访问控制（RBAC）

---

## 2. 技术架构

### 2.1 整体架构

采用**前后端完全分离**的架构：

```
┌─────────────────┐         HTTP/REST          ┌─────────────────┐
│                 │ ◄─────────────────────────► │                 │
│  Web Console    │                             │  Artemis Server │
│  (React SPA)    │         WebSocket           │  (Rust Backend) │
│                 │ ◄─────────────────────────► │                 │
└─────────────────┘                             └─────────────────┘
      │                                                  │
      │ Deploy                                           │ Deploy
      ▼                                                  ▼
  Nginx/CDN                                      Kubernetes/Docker
```

### 2.2 技术栈

| 技术 | 选型 | 版本 | 用途 |
|------|------|------|------|
| **基础框架** | React | 18.x | UI 框架 |
| **类型系统** | TypeScript | 5.x | 类型安全 |
| **构建工具** | Vite | 5.x | 开发/构建 |
| **UI 组件库** | Material-UI (MUI) | 5.x | UI 组件 |
| **路由** | React Router | 6.x | 路由管理 |
| **状态管理** | Zustand | 4.x | 全局状态（轻量级） |
| **HTTP 客户端** | Axios | 1.x | API 请求 |
| **WebSocket** | Native WebSocket | - | 实时通信 |
| **图表** | Recharts | 2.x | 数据可视化 |
| **表单** | React Hook Form | 7.x | 表单处理 |
| **日期处理** | date-fns | 3.x | 日期格式化 |
| **代码规范** | ESLint + Prettier | - | 代码质量 |

### 2.3 项目结构

```
artemis-console/                 # 前端项目根目录
├── public/                      # 静态资源
│   ├── favicon.ico
│   └── logo.png
├── src/
│   ├── main.tsx                # 应用入口
│   ├── App.tsx                 # 根组件
│   ├── vite-env.d.ts          # Vite 类型定义
│   │
│   ├── api/                    # API 调用层
│   │   ├── client.ts          # Axios 实例配置
│   │   ├── types.ts           # API 请求/响应类型
│   │   ├── registry.ts        # 注册 API
│   │   ├── discovery.ts       # 发现 API
│   │   ├── management.ts      # 管理 API
│   │   ├── routing.ts         # 路由 API
│   │   ├── cluster.ts         # 集群 API
│   │   ├── audit.ts           # 审计日志 API
│   │   ├── canary.ts          # 金丝雀 API
│   │   ├── zone.ts            # Zone 操作 API
│   │   └── auth.ts            # 认证/权限 API
│   │
│   ├── components/             # 通用组件
│   │   ├── Layout/            # 布局组件
│   │   │   ├── MainLayout.tsx
│   │   │   ├── Sidebar.tsx
│   │   │   └── Header.tsx
│   │   ├── DataTable/         # 数据表格
│   │   ├── Charts/            # 图表组件
│   │   ├── StatusBadge/       # 状态标签
│   │   └── WebSocketStatus/   # WebSocket 连接状态
│   │
│   ├── pages/                  # 页面组件
│   │   ├── Dashboard/         # 仪表板
│   │   ├── Services/          # 服务管理
│   │   ├── Instances/         # 实例管理
│   │   ├── Cluster/           # 集群管理
│   │   ├── Routing/           # 路由配置
│   │   ├── AuditLog/          # 审计日志
│   │   ├── ZoneOps/           # Zone 批量操作
│   │   ├── Canary/            # 金丝雀发布
│   │   ├── Users/             # 用户管理
│   │   └── Login/             # 登录页
│   │
│   ├── hooks/                  # 自定义 Hooks
│   │   ├── useWebSocket.ts    # WebSocket 钩子
│   │   ├── usePolling.ts      # 轮询钩子
│   │   ├── useAuth.ts         # 认证钩子
│   │   └── useNotification.ts # 通知钩子
│   │
│   ├── store/                  # 状态管理（Zustand）
│   │   ├── authStore.ts       # 认证状态
│   │   ├── servicesStore.ts   # 服务状态
│   │   ├── clusterStore.ts    # 集群状态
│   │   └── uiStore.ts         # UI 状态
│   │
│   ├── routes/                 # 路由配置
│   │   └── index.tsx
│   │
│   ├── utils/                  # 工具函数
│   │   ├── format.ts          # 格式化工具
│   │   ├── validators.ts      # 校验工具
│   │   └── constants.ts       # 常量定义
│   │
│   └── theme/                  # 主题配置
│       └── index.ts
│
├── package.json
├── tsconfig.json
├── vite.config.ts
├── .eslintrc.cjs
├── .prettierrc
└── README.md
```

---

## 3. 功能模块设计

### 3.1 仪表板（Dashboard）

**功能概览**：
- 集群整体健康状态（节点数、在线/离线）
- 服务实例统计（总数、UP/DOWN/UNHEALTHY 分布）
- 实时请求 QPS 图表（注册、心跳、发现）
- 最近操作日志（TOP 10）

**核心组件**：
- `ClusterHealthCard` - 集群健康卡片
- `ServiceStatsCard` - 服务统计卡片
- `QpsChart` - QPS 趋势图表（Recharts 折线图）
- `RecentActivityList` - 最近活动列表

**数据获取**：
- 定时轮询（30秒）获取统计数据
- WebSocket 接收实时事件更新

---

### 3.2 服务管理（Services）

**功能列表**：
- 服务列表展示（表格，支持分页、排序）
- 多维度搜索（service_id、region_id、zone_id）
- 服务详情（实例列表、分组信息、路由规则）
- 服务实例筛选（按状态、IP、分组）
- 批量操作（导出服务列表）

**页面结构**：
```
/services
  ├── ServiceList (列表页)
  └── /services/:serviceId (详情页)
      ├── InstancesTab (实例列表)
      ├── GroupsTab (分组信息)
      └── RoutingTab (路由规则)
```

**核心组件**：
- `ServiceTable` - 服务列表表格（MUI DataGrid）
- `ServiceSearchBar` - 搜索栏（region/zone/service 级联选择）
- `InstanceStatusPie` - 实例状态饼图
- `ServiceDetailDrawer` - 服务详情抽屉

**API 对接**：
- `POST /api/discovery/services.json` - 获取服务列表
- `POST /api/discovery/service.json` - 获取服务详情

---

### 3.3 实例管理（Instances）

**功能列表**：
- 实例全局视图（跨服务查询）
- 实例状态变更操作（拉入/拉出）
- 服务器批量操作（按 IP 拉出所有实例）
- 实例元数据查看/编辑

**操作流程**：
```
拉出实例:
1. 选择实例 → 2. 确认操作 → 3. 调用 API → 4. WebSocket 接收变更 → 5. 刷新列表
```

**核心组件**：
- `InstanceTable` - 实例列表（支持多选）
- `InstanceOperationDialog` - 操作确认对话框
- `ServerBatchOperation` - 服务器批量操作面板
- `InstanceMetadataEditor` - 元数据编辑器（JSON 编辑）

**API 对接**：
- `POST /api/management/instance/operate-instance.json` - 实例操作
- `POST /api/management/server/operate-server.json` - 服务器操作
- `POST /api/management/instance/get-instance-operations.json` - 查询操作历史

---

### 3.4 集群管理（Cluster）

**功能列表**：
- 集群节点列表（节点 URL、状态、最后心跳时间）
- 节点健康检查（手动触发/自动轮询）
- 数据复制状态监控（复制队列长度、失败次数）

**核心组件**：
- `ClusterNodeTable` - 节点列表表格
- `NodeHealthIndicator` - 节点健康指示器
- `ReplicationQueueChart` - 复制队列图表

**API 对接**：
- `GET /api/status/cluster` - 获取集群状态
- `GET /api/status/cluster/nodes` - 获取节点状态

---

### 3.5 路由配置（Routing）

**功能列表**：
- 服务分组管理（创建/编辑/删除分组）
- 路由规则配置（加权轮询、就近访问）
- 分组权重可视化配置（拖拽滑块调整权重）
- 路由规则发布（预览 → 发布 → 回滚）
- 路由效果预览（模拟请求分配）

**页面结构**：
```
/routing
  ├── /groups (分组管理)
  └── /rules (路由规则)
      └── /rules/:ruleId/edit (规则编辑器)
```

**核心组件**：
- `GroupManager` - 分组管理面板
- `RouteRuleEditor` - 路由规则编辑器
- `WeightSlider` - 权重滑块组件
- `RouteSimulator` - 路由模拟器（显示分配比例）
- `RulePublishDialog` - 发布确认对话框

**API 对接**：
- `GET /api/routing/groups` - 获取分组列表
- `POST /api/routing/groups` - 创建分组
- `GET /api/routing/rules` - 获取路由规则
- `POST /api/routing/rules/:ruleId/publish` - 发布规则

---

### 3.6 审计日志（Audit Log）

**功能列表**：
- 日志查询（时间范围、操作类型、操作人、服务 ID）
- 日志详情展示（操作前后状态对比）
- 日志导出（CSV/JSON 格式）

**查询条件**：
```typescript
interface AuditLogQuery {
  startTime: Date;
  endTime: Date;
  operationType?: 'register' | 'unregister' | 'pull_in' | 'pull_out' | 'route_publish';
  operatorId?: string;
  serviceId?: string;
  instanceId?: string;
  pageSize: number;
  pageNum: number;
}
```

**核心组件**：
- `AuditLogTable` - 日志表格（带高级筛选）
- `AuditLogFilter` - 多条件筛选器
- `OperationDiffViewer` - 操作前后对比视图（JSON Diff）
- `LogExportDialog` - 导出配置对话框

**API 对接**：
- `GET /api/audit/logs` - 查询审计日志
- `GET /api/audit/instance-logs` - 查询实例操作日志
- `GET /api/audit/server-logs` - 查询服务器操作日志

---

### 3.7 Zone 批量操作（Zone Operations）

**功能列表**：
- Zone 级别实例查看（按 Zone 分组显示所有实例）
- Zone 批量拉出/拉入（一键操作整个可用区）
- 操作历史记录（Zone 级别的操作日志）
- 操作影响预估（显示将影响的服务和实例数量）

**操作流程**：
```
Zone 批量拉出:
1. 选择 Zone
2. 预览影响范围（N 个服务，M 个实例）
3. 确认操作 + 填写原因
4. 执行批量操作
5. 显示进度（已完成/失败实例）
6. 记录审计日志
```

**核心组件**：
- `ZoneSelector` - Zone 选择器（树形结构：Region → Zone）
- `ZoneImpactPreview` - 影响预估面板
- `BatchOperationProgress` - 批量操作进度条
- `ZoneOperationHistory` - 操作历史表格

**API 对接**：
- `POST /api/management/zones/pull-out` - 拉出 Zone
- `POST /api/management/zones/pull-in` - 拉入 Zone
- `GET /api/management/zones/:zoneId/status` - 获取 Zone 状态
- `GET /api/management/zones/operations` - 操作历史

---

### 3.8 金丝雀发布（Canary Deployment）

**功能列表**：
- 金丝雀配置管理（服务白名单、流量比例）
- 灰度发布流程（0% → 5% → 25% → 50% → 100%）
- 实时监控（金丝雀实例 vs 生产实例的对比）
- 发布历史和版本管理

**发布流程**：
```
1. 创建金丝雀配置
   - 选择服务
   - 配置白名单（IP 列表）
   - 设置初始流量（如 5%）

2. 发布金丝雀版本
   - 注册金丝雀实例（带特殊 metadata）
   - 应用路由规则

3. 监控和调整
   - 查看金丝雀指标
   - 逐步增加流量（5% → 25% → 50%）

4. 全量发布或回滚
   - 全量：流量 100%，下线旧实例
   - 回滚：流量 0%，恢复原路由规则
```

**核心组件**：
- `CanaryConfigEditor` - 金丝雀配置编辑器
- `TrafficSlider` - 流量比例滑块
- `WhitelistManager` - 白名单管理（IP 列表输入/导入）
- `CanaryReleaseTimeline` - 发布时间线

**API 对接**：
- `GET /api/canary/configs` - 获取配置列表
- `POST /api/canary/config` - 创建配置
- `POST /api/canary/enable` - 启用金丝雀
- `DELETE /api/canary/config` - 删除配置

---

### 3.9 用户权限管理（User Management）

**功能列表**：
- 用户账号管理（创建/编辑/禁用用户）
- 角色管理（Admin、Operator、Viewer）
- 权限控制（基于角色的 RBAC）
- 操作审计（记录用户所有操作）

**权限模型**：
```typescript
enum Role {
  Admin = 'admin',       // 所有权限
  Operator = 'operator', // 可管理实例、配置路由，不能管理用户
  Viewer = 'viewer',     // 只读权限
}

interface Permission {
  resource: 'service' | 'instance' | 'cluster' | 'routing' | 'audit' | 'user';
  actions: ('read' | 'write' | 'delete')[];
}
```

**核心组件**：
- `UserTable` - 用户列表
- `UserEditDialog` - 用户编辑对话框
- `RolePermissionMatrix` - 权限矩阵（表格展示角色和权限）
- `PermissionGuard` - 权限守卫组件（HOC）

**API 对接**（需要后端新增）：
- `POST /api/auth/login` - 登录
- `POST /api/auth/logout` - 登出
- `GET /api/auth/me` - 获取当前用户信息
- `GET /api/users` - 获取用户列表
- `POST /api/users` - 创建用户
- `PUT /api/users/:userId` - 更新用户信息
- `GET /api/users/:userId/permissions` - 获取用户权限

---

## 4. 数据流和实时通信

### 4.1 WebSocket 实时更新机制

**实时更新场景**：
1. **服务实例变更**：注册/注销/状态变化 → 自动更新服务列表和实例列表
2. **集群节点变更**：节点上线/下线 → 更新集群节点列表
3. **路由规则发布**：规则变更 → 刷新路由配置页面
4. **操作进度通知**：批量操作进度 → 实时显示进度条

**WebSocket 架构**：
```typescript
// WebSocket 连接管理
class WebSocketManager {
  private ws: WebSocket | null = null;
  private reconnectTimer: NodeJS.Timeout | null = null;
  private subscribers: Map<string, Set<(data: any) => void>> = new Map();

  connect(url: string) {
    this.ws = new WebSocket(url);

    this.ws.onopen = () => {
      console.log('WebSocket connected');
      this.clearReconnectTimer();
    };

    this.ws.onmessage = (event) => {
      const message = JSON.parse(event.data);
      this.notify(message.type, message.data);
    };

    this.ws.onclose = () => {
      console.log('WebSocket disconnected');
      this.scheduleReconnect();
    };
  }

  // 订阅特定类型的消息
  subscribe(type: string, callback: (data: any) => void) {
    if (!this.subscribers.has(type)) {
      this.subscribers.set(type, new Set());
    }
    this.subscribers.get(type)!.add(callback);
  }

  // 通知订阅者
  private notify(type: string, data: any) {
    this.subscribers.get(type)?.forEach(callback => callback(data));
  }

  // 自动重连
  private scheduleReconnect() {
    this.reconnectTimer = setTimeout(() => {
      this.connect(this.ws!.url);
    }, 5000);
  }
}
```

**消息类型定义**：
```typescript
type WebSocketMessage =
  | { type: 'instance_change', data: { serviceId: string, instances: Instance[] } }
  | { type: 'cluster_node_change', data: { nodes: ClusterNode[] } }
  | { type: 'route_rule_change', data: { ruleId: string, rule: RouteRule } }
  | { type: 'batch_operation_progress', data: { operationId: string, progress: number } };
```

**使用示例**：
```typescript
// 在服务列表页面订阅实例变更
const useServiceUpdates = (serviceId: string) => {
  const [service, setService] = useState<Service | null>(null);

  useEffect(() => {
    const ws = getWebSocketManager();

    ws.subscribe('instance_change', (data) => {
      if (data.serviceId === serviceId) {
        setService(prev => ({ ...prev, instances: data.instances }));
      }
    });

    return () => {
      // 清理订阅
    };
  }, [serviceId]);

  return service;
};
```

---

### 4.2 API 调用层设计

**Axios 实例配置**：
```typescript
// src/api/client.ts
import axios from 'axios';

const apiClient = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080',
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// 请求拦截器：添加 Token
apiClient.interceptors.request.use((config) => {
  const token = localStorage.getItem('auth_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// 响应拦截器：错误处理
apiClient.interceptors.response.use(
  (response) => response.data,
  (error) => {
    if (error.response?.status === 401) {
      // 未授权，跳转登录
      window.location.href = '/login';
    }
    return Promise.reject(error);
  }
);
```

**API 模块示例**（Discovery API）：
```typescript
// src/api/discovery.ts
import { apiClient } from './client';
import type { Service, GetServiceRequest, GetServicesResponse } from './types';

export const discoveryApi = {
  // 获取单个服务
  async getService(request: GetServiceRequest): Promise<Service> {
    return apiClient.post('/api/discovery/service.json', {
      discovery_config: {
        service_id: request.serviceId,
        region_id: request.regionId,
        zone_id: request.zoneId,
      }
    });
  },

  // 获取所有服务
  async getAllServices(regionId: string): Promise<GetServicesResponse> {
    return apiClient.post('/api/discovery/services.json', {
      discovery_config: {
        region_id: regionId,
      }
    });
  },
};
```

---

### 4.3 状态管理（Zustand）

**全局状态结构**：
```typescript
// src/store/authStore.ts
import create from 'zustand';

interface AuthState {
  user: User | null;
  token: string | null;
  permissions: Permission[];

  login: (username: string, password: string) => Promise<void>;
  logout: () => void;
  checkPermission: (resource: string, action: string) => boolean;
}

export const useAuthStore = create<AuthState>((set, get) => ({
  user: null,
  token: localStorage.getItem('auth_token'),
  permissions: [],

  login: async (username, password) => {
    const response = await authApi.login(username, password);
    localStorage.setItem('auth_token', response.token);
    set({ user: response.user, token: response.token, permissions: response.permissions });
  },

  logout: () => {
    localStorage.removeItem('auth_token');
    set({ user: null, token: null, permissions: [] });
  },

  checkPermission: (resource, action) => {
    const { permissions } = get();
    return permissions.some(p => p.resource === resource && p.actions.includes(action));
  },
}));
```

```typescript
// src/store/servicesStore.ts
import create from 'zustand';

interface ServicesState {
  services: Map<string, Service>;
  loading: boolean;

  fetchServices: (regionId: string) => Promise<void>;
  updateService: (serviceId: string, service: Service) => void;
}

export const useServicesStore = create<ServicesState>((set, get) => ({
  services: new Map(),
  loading: false,

  fetchServices: async (regionId) => {
    set({ loading: true });
    const response = await discoveryApi.getAllServices(regionId);
    const servicesMap = new Map(response.services.map(s => [s.service_id, s]));
    set({ services: servicesMap, loading: false });
  },

  updateService: (serviceId, service) => {
    set((state) => {
      const newServices = new Map(state.services);
      newServices.set(serviceId, service);
      return { services: newServices };
    });
  },
}));
```

---

## 5. 后端 API 对接

### 5.1 现有 API 盘点

根据 Artemis 后端代码审查，已实现以下管理 API：

#### ✅ 实例管理 API
```
POST /api/management/instance/operate-instance.json      - 实例拉入/拉出
POST /api/management/instance/get-instance-operations.json - 查询实例操作列表
POST /api/management/instance/is-instance-down.json      - 查询实例是否被拉出
```

#### ✅ 服务器管理 API
```
POST /api/management/server/operate-server.json          - 服务器批量操作
POST /api/management/server/is-server-down.json         - 查询服务器是否被拉出
GET  /api/management/server/get-all-operations          - 获取所有服务器操作
```

#### ✅ 分组路由 API
```
POST   /api/routing/groups                               - 创建分组
GET    /api/routing/groups                               - 列出分组
GET    /api/routing/groups/:groupId                      - 获取分组详情
PUT    /api/routing/groups/:groupId                      - 更新分组
DELETE /api/routing/groups/:groupId                      - 删除分组
POST   /api/routing/groups/:groupId/instances           - 添加实例到分组
GET    /api/routing/groups/:groupId/instances           - 获取分组实例列表
POST   /api/routing/rules                                - 创建路由规则
GET    /api/routing/rules                                - 列出路由规则
POST   /api/routing/rules/:ruleId/publish               - 发布路由规则
POST   /api/routing/rules/:ruleId/groups                - 添加分组到规则
```

#### ✅ Zone 管理 API
```
POST   /api/management/zones/pull-out                    - 拉出 Zone
POST   /api/management/zones/pull-in                     - 拉入 Zone
GET    /api/management/zones/:zoneId/status             - 获取 Zone 状态
GET    /api/management/zones/operations                  - 列出 Zone 操作
DELETE /api/management/zones/operations/:operationId    - 删除 Zone 操作
```

#### ✅ 金丝雀发布 API
```
POST   /api/canary/config                                - 设置金丝雀配置
GET    /api/canary/config                                - 获取金丝雀配置
POST   /api/canary/enable                                - 启用金丝雀
DELETE /api/canary/config                                - 删除金丝雀配置
GET    /api/canary/configs                               - 列出所有金丝雀配置
```

#### ✅ 审计日志 API
```
GET /api/audit/logs                                       - 查询审计日志
GET /api/audit/instance-logs                             - 查询实例操作日志
GET /api/audit/server-logs                               - 查询服务器操作日志
GET /api/audit/group-logs                                - 查询分组操作日志
GET /api/audit/route-rule-logs                           - 查询路由规则日志
GET /api/audit/zone-operation-logs                       - 查询 Zone 操作日志
```

#### ✅ 集群状态 API
```
GET /api/status/cluster                                   - 获取集群状态
GET /api/status/cluster/nodes                            - 获取集群节点状态
```

#### ✅ 服务发现 API
```
POST /api/discovery/services.json                         - 获取服务列表
POST /api/discovery/service.json                          - 获取服务详情
POST /api/discovery/services/delta.json                   - 增量同步
```

#### ✅ WebSocket API
```
WS /api/v1/discovery/subscribe/:serviceId                - 订阅服务变更通知
```

---

### 5.2 需要新增的 API（仅用户权限相关）

| 功能 | API 端点 | 说明 |
|------|----------|------|
| **认证** | `POST /api/auth/login` | 用户登录 |
| | `POST /api/auth/logout` | 用户登出 |
| | `GET /api/auth/me` | 获取当前用户信息 |
| **用户管理** | `GET /api/users` | 获取用户列表 |
| | `POST /api/users` | 创建用户 |
| | `GET /api/users/:userId` | 获取用户详情 |
| | `PUT /api/users/:userId` | 更新用户信息 |
| | `DELETE /api/users/:userId` | 删除用户 |
| | `PUT /api/users/:userId/password` | 修改密码 |
| **权限管理** | `GET /api/roles` | 获取角色列表 |
| | `GET /api/users/:userId/permissions` | 获取用户权限 |
| | `PUT /api/users/:userId/role` | 更新用户角色 |

---

### 5.3 前端功能与 API 映射

| 前端功能 | 使用的 API | 说明 |
|---------|-----------|------|
| **仪表板统计** | `GET /api/status/cluster` + 轮询 `/api/discovery/services.json` | 从集群状态和服务列表计算统计数据 |
| **服务列表** | `POST /api/discovery/services.json` | ✅ 已有 |
| **服务详情** | `POST /api/discovery/service.json` | ✅ 已有 |
| **实例操作** | `POST /api/management/instance/operate-instance.json` | ✅ 已有 |
| **实例操作历史** | `POST /api/management/instance/get-instance-operations.json` | ✅ 已有 |
| **服务器批量操作** | `POST /api/management/server/operate-server.json` | ✅ 已有 |
| **集群节点状态** | `GET /api/status/cluster/nodes` | ✅ 已有 |
| **分组管理** | `/api/routing/groups/*` | ✅ 已有 |
| **路由规则** | `/api/routing/rules/*` | ✅ 已有 |
| **Zone 操作** | `/api/management/zones/*` | ✅ 已有 |
| **金丝雀发布** | `/api/canary/*` | ✅ 已有 |
| **审计日志** | `/api/audit/*` | ✅ 已有 |
| **用户权限** | `/api/auth/*`, `/api/users/*` | ❌ 需要新增 |

---

## 6. UI/UX 设计

### 6.1 整体布局

```
┌─────────────────────────────────────────────────────┐
│  Header (Logo + User Menu + Notifications)         │
├────────┬────────────────────────────────────────────┤
│        │                                            │
│ Side   │                                            │
│ bar    │           Main Content Area               │
│        │                                            │
│ - 仪表板 │                                            │
│ - 服务  │                                            │
│ - 实例  │                                            │
│ - 集群  │                                            │
│ - 路由  │                                            │
│ - 审计  │                                            │
│ - Zone │                                            │
│ - 金丝雀 │                                            │
│ - 用户  │                                            │
│        │                                            │
└────────┴────────────────────────────────────────────┘
```

### 6.2 Material Design 主题配置

```typescript
// src/theme/index.ts
import { createTheme } from '@mui/material/styles';

export const lightTheme = createTheme({
  palette: {
    mode: 'light',
    primary: {
      main: '#1976d2',  // 蓝色主色调
    },
    secondary: {
      main: '#dc004e',  // 红色强调色
    },
    success: {
      main: '#4caf50',  // 绿色（UP 状态）
    },
    warning: {
      main: '#ff9800',  // 橙色（STARTING 状态）
    },
    error: {
      main: '#f44336',  // 红色（DOWN 状态）
    },
  },
  typography: {
    fontFamily: '"Roboto", "Helvetica", "Arial", sans-serif',
  },
});

export const darkTheme = createTheme({
  palette: {
    mode: 'dark',
    primary: {
      main: '#90caf9',
    },
    secondary: {
      main: '#f48fb1',
    },
  },
});
```

### 6.3 实例状态颜色映射

```typescript
const statusColors = {
  up: 'success',      // 绿色
  down: 'error',      // 红色
  starting: 'warning', // 橙色
  unhealthy: 'warning', // 橙色
  unknown: 'default',  // 灰色
};
```

### 6.4 响应式设计

- **桌面端（≥1280px）**：侧边栏展开，显示完整导航文字
- **平板端（768px - 1279px）**：侧边栏收起，仅显示图标
- **移动端（<768px）**：侧边栏隐藏，通过菜单按钮展开

### 6.5 交互设计

- **加载状态**：使用 Skeleton 占位符，避免空白闪烁
- **错误处理**：全局 Snackbar 提示错误信息
- **确认操作**：危险操作（拉出实例、删除配置）使用确认对话框
- **表格分页**：默认每页 20 条，支持 10/20/50/100 切换
- **搜索优化**：防抖延迟 300ms，避免频繁请求

---

## 7. 实施计划

采用**全功能并行开发**策略，分为以下阶段：

### 第 1 周：项目搭建和基础设施

**任务**：
- 初始化 React + TypeScript + Vite 项目
- 配置 Material-UI、React Router、Zustand
- 搭建项目目录结构
- 实现基础布局组件（MainLayout、Sidebar、Header）
- 配置 Axios 实例和 API 层架构
- 实现路由配置和权限守卫
- 配置 ESLint、Prettier、Git hooks

**交付物**：
- 可运行的空白控制台框架
- 登录页面（UI only，无后端集成）
- 基础导航和路由跳转

---

### 第 2-4 周：核心功能模块并行开发

按照模块分工并行开发所有功能：

**第 2 周：浏览类功能**
- ✅ 仪表板（Dashboard）- 统计卡片、QPS 图表
- ✅ 服务管理（Services）- 列表、详情、搜索
- ✅ 实例管理（Instances）- 全局实例视图
- ✅ 集群管理（Cluster）- 节点列表、健康检查

**第 3 周：操作类功能**
- ✅ 实例操作（拉入/拉出）- 操作对话框、历史记录
- ✅ 路由配置（Routing）- 分组管理、规则编辑、权重滑块
- ✅ Zone 批量操作 - 影响预估、批量操作面板

**第 4 周：高级功能**
- ✅ 金丝雀发布（Canary）- 配置编辑器、流量调整
- ✅ 审计日志（Audit）- 多条件查询、详情展示
- ✅ 用户权限（Users）- 用户列表、角色管理、权限矩阵

---

### 第 5 周：集成和优化

**任务**：
- WebSocket 实时更新集成
- API 错误处理和重试机制
- 全局加载状态优化
- 响应式布局调整（移动端适配）
- 性能优化（代码分割、懒加载）
- 国际化支持（i18n，可选）

**交付物**：
- 完整功能的 Web 控制台
- 所有模块集成测试通过

---

### 第 6 周：测试、文档和发布

**任务**：
- 端到端测试（E2E）
- 跨浏览器兼容性测试
- 性能测试（Lighthouse）
- 用户文档编写
- 部署配置（Dockerfile、nginx.conf）
- 生产环境部署

**交付物**：
- 生产就绪的控制台应用
- 用户手册和开发文档
- Docker 镜像

---

## 8. 风险评估

| 风险 | 影响 | 概率 | 缓解措施 |
|------|------|------|---------|
| **后端 API 不完善** | 高 | 中 | - 优先对接现有 API<br>- 前端先用 Mock 数据开发<br>- 与后端团队提前沟通用户认证 API |
| **WebSocket 连接不稳定** | 中 | 低 | - 实现自动重连机制<br>- 降级方案：轮询 API |
| **性能问题（大数据量）** | 中 | 中 | - 虚拟滚动（react-window）<br>- 前端分页<br>- 数据缓存策略 |
| **浏览器兼容性** | 低 | 低 | - 使用 Vite 的 legacy 插件<br>- 测试主流浏览器（Chrome、Firefox、Safari） |
| **安全漏洞（XSS、CSRF）** | 高 | 低 | - 使用 React 默认转义<br>- CSRF Token 验证<br>- Content Security Policy |
| **开发进度延期** | 中 | 中 | - 每周 checkpoint 评估进度<br>- MVP 优先策略（可削减部分功能） |

---

## 9. 成功标准

### 9.1 功能完整性
- ✅ 9 个核心模块全部实现
- ✅ 支持所有后端已有的管理 API
- ✅ 实时 WebSocket 更新正常工作
- ✅ 用户权限控制生效

### 9.2 性能指标
- ✅ 首屏加载时间 < 2 秒（3G 网络）
- ✅ 页面交互响应 < 100ms
- ✅ Lighthouse 性能评分 > 90

### 9.3 用户体验
- ✅ 响应式设计，支持桌面端和平板端
- ✅ 深色/浅色主题切换
- ✅ 错误提示清晰友好
- ✅ 操作有加载提示和成功反馈

### 9.4 代码质量
- ✅ TypeScript 类型覆盖率 > 95%
- ✅ ESLint 零警告
- ✅ 关键功能有单元测试（可选）

---

## 10. 部署配置

### 10.1 Vite 构建配置

```typescript
// vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  plugins: [react()],
  server: {
    port: 3000,
    proxy: {
      '/api': {
        target: 'http://localhost:8080',
        changeOrigin: true,
      },
    },
  },
  build: {
    outDir: 'dist',
    sourcemap: true,
    rollupOptions: {
      output: {
        manualChunks: {
          'react-vendor': ['react', 'react-dom', 'react-router-dom'],
          'mui': ['@mui/material', '@mui/icons-material'],
          'charts': ['recharts'],
        },
      },
    },
  },
});
```

### 10.2 环境变量

```bash
# .env.development
VITE_API_BASE_URL=http://localhost:8080
VITE_WS_BASE_URL=ws://localhost:8080

# .env.production
VITE_API_BASE_URL=https://artemis.example.com
VITE_WS_BASE_URL=wss://artemis.example.com
```

### 10.3 Docker 部署

```dockerfile
# Dockerfile
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM nginx:alpine
COPY --from=builder /app/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/conf.d/default.conf
EXPOSE 80
CMD ["nginx", "-g", "daemon off;"]
```

```nginx
# nginx.conf
server {
    listen 80;
    server_name localhost;
    root /usr/share/nginx/html;
    index index.html;

    # SPA 路由支持
    location / {
        try_files $uri $uri/ /index.html;
    }

    # API 代理（可选）
    location /api {
        proxy_pass http://artemis-backend:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }

    # WebSocket 代理（可选）
    location /api/v1/discovery/subscribe {
        proxy_pass http://artemis-backend:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }

    # Gzip 压缩
    gzip on;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml application/xml+rss text/javascript;
}
```

---

## 11. 后续扩展规划（可选）

1. **国际化**：支持中文、英文双语切换
2. **移动端 App**：使用 React Native 或 PWA 技术
3. **高级监控**：集成 Grafana 仪表板
4. **自动化运维**：定时任务、批量脚本执行
5. **插件系统**：支持自定义页面和功能扩展

---

## 12. 设计总结

### 12.1 核心亮点

1. **技术栈现代化**：React 18 + TypeScript + Material-UI，企业级标准
2. **架构清晰**：前后端分离，职责明确，易于维护
3. **功能完整**：覆盖服务治理全生命周期（注册、发现、路由、监控、审计）
4. **用户体验优秀**：Material Design 规范，交互流畅，实时更新
5. **扩展性强**：模块化设计，易于添加新功能

### 12.2 关键决策

- ✅ 前后端完全分离（独立部署）
- ✅ 使用现有后端 API，仅新增用户认证模块
- ✅ 全功能并行开发（6 周交付）
- ✅ Material-UI 组件库（国际化优先）
- ✅ Zustand 状态管理（轻量级）

---

**文档结束**
