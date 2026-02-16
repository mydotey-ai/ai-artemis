# Cluster 页面实现

**文档状态**: ✅ 最新
**最后更新**: 2026-02-17
**相关 Phase**: Phase 2
**源代码**: `artemis-console/src/pages/Cluster/Cluster.tsx`

---

## 概述

Artemis Console 的 Cluster 页面完整实现文档，包含集群拓扑可视化、节点列表、统计信息和节点详情等功能。

## 功能清单

### 1. 集群统计卡片 ✅
- **Total Nodes**: 显示集群节点总数
- **Healthy Nodes**: 显示健康节点数量（状态为 ACTIVE）
- **Total Instances**: 显示所有节点托管的实例总数
- **Total Services**: 显示所有服务总数

### 2. 集群拓扑可视化 ✅
- **SVG 绘制**: 使用原生 SVG 绘制，无需第三方图形库
- **圆形布局**: 节点排列成圆形，直观展示集群结构
- **节点状态颜色**:
  - 绿色 (#4caf50): ACTIVE（活跃）
  - 橙色 (#ff9800): SUSPECTED（疑似故障）
  - 红色 (#f44336): INACTIVE（不活跃）
- **连接线**: 虚线连接所有节点，表示复制关系
- **交互功能**: 点击节点可查看详情
- **响应式设计**: 自适应容器宽度
- **图例**: 显示状态颜色说明

### 3. 集群节点列表 ✅
- **表格列**:
  - Node ID: 节点唯一标识（等宽字体）
  - Host: 主机地址
  - Port: 端口号
  - Status: 状态（带颜色徽章和图标）
  - Region / Zone: 区域和可用区
  - Last Heartbeat: 最后心跳时间（相对时间格式）
  - Actions: 操作按钮
- **状态徽章**: 不同状态使用不同颜色和图标
- **表格交互**: 行悬停效果
- **空状态处理**: 无节点时显示提示信息

### 4. 节点详情对话框 ✅
- **基本信息**:
  - Node ID
  - Host
  - Port
  - Status（带徽章）
  - Region ID
  - Zone ID
  - Last Heartbeat（绝对时间 + 相对时间）
- **节点 URL**: 显示完整的 HTTP URL
- **响应式设计**: 使用 Material-UI Dialog

### 5. 实时更新 ✅
- **自动刷新**: 每 5 秒自动刷新集群状态
- **手动刷新**: 顶部刷新按钮
- **更新时间显示**: 显示最后更新时间

### 6. 错误处理 ✅
- **错误提示**: API 调用失败时显示 Alert
- **加载状态**: 显示 Loading 指示器
- **Snackbar 通知**: 操作成功/失败提示（预留）

### 7. 响应式设计 ✅
- **统计卡片**: xs=12, sm=6, md=3（移动端单列，平板双列，桌面四列）
- **拓扑图**: 自适应容器宽度，横向滚动
- **表格**: 移动端横向滚动

## 技术实现

### API 集成
使用现有的 cluster API：
```typescript
import { getClusterStatus, getClusterNodeStatus } from '@/api/cluster';
```

- `getClusterNodeStatus()`: 获取所有节点状态
- `getClusterStatus()`: 获取集群整体统计

### 类型安全
- 使用 TypeScript 严格模式
- 使用 type-only imports
- 所有组件和函数都有完整的类型定义

### 性能优化
- 使用 `useMemo` 缓存拓扑组件
- 使用 `useCallback` 缓存回调函数
- 避免不必要的重新渲染

### 代码规范
- 符合 ESLint 规则
- 移除了未使用的导入（SpeedIcon, ApiResponse, NodeStatus）
- 所有函数和组件都有 JSDoc 注释

## 文件位置

```
artemis-console/src/pages/Cluster/Cluster.tsx
```

## 代码统计

- **总行数**: 756 行
- **组件**: 1 个主组件 + 1 个子组件（ClusterTopology）
- **State 变量**: 7 个
- **Hooks**: useEffect, useState, useCallback, useMemo
- **Material-UI 组件**: 30+ 个

## 未实现功能

由于后端 API 限制，以下功能未实现：

1. **添加节点**: 需要 `registerNode(nodeUrl)` API
2. **注销节点**: 需要 `deregisterNode(nodeId)` API
3. **复制队列详情**: 需要 API 返回每个节点的待复制数量

这些功能可以在后端提供相应 API 后轻松添加。

## 测试建议

1. 启动集群：`./scripts/cluster.sh start`
2. 访问 Console: `http://localhost:3000/cluster`
3. 验证功能：
   - 查看统计卡片是否正确显示
   - 拓扑图是否正确渲染节点和连接线
   - 点击节点是否打开详情对话框
   - 表格是否显示所有节点信息
   - 自动刷新是否正常工作

## 截图说明

页面包含以下部分：
1. 顶部：页面标题 + 最后更新时间 + 刷新按钮
2. 统计卡片：4 个彩色卡片（蓝、绿、橙、紫）
3. 拓扑图：SVG 圆形布局，节点用圆圈表示，带状态颜色
4. 节点列表：Material-UI 表格，显示所有节点信息

## 下一步

如果需要添加节点管理功能，需要：

1. 后端实现节点注册/注销 API
2. 前端添加 "Add Node" 按钮和 Dialog
3. 前端添加节点操作菜单（Deregister 按钮）
4. 添加确认对话框和错误处理

---

**实现时间**: 2026-02-17
**开发者**: Claude Sonnet 4.5
