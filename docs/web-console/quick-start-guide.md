# Artemis Web Console 测试指南

## 🚀 快速开始

### 1. 启动服务

```bash
# 一键启动开发环境 (推荐)
./scripts/dev.sh start

# 或手动启动
# 后端: cargo run --release --bin artemis -- server
# 前端: cd artemis-console && npm run dev
```

**服务地址**:
- 后端 API: http://localhost:8080
- Web 控制台: http://localhost:5173
- 健康检查: http://localhost:8080/health

### 2. 登录 Web Console

在浏览器中打开: **http://localhost:5173**

**默认登录凭据**:
- 用户名: `admin`
- 密码: `admin123`
- 角色: 管理员

> ⚠️ **首次登录后请立即修改密码！**

### 3. 准备测试数据（可选）

运行测试脚本注册服务实例：

```bash
./scripts/test-all-operations.sh
```

---

## 📊 测试数据概览

已注册的测试服务:

1. **user-service** (2 个实例)
   - `user-1`: 192.168.1.100:8080 - UP - version: 1.0.0
   - `user-2`: 192.168.1.101:8080 - UP - version: 1.0.0

2. **order-service** (3 个实例)
   - `order-1`: 192.168.1.200:8081 - UP - version: 2.0.0
   - `order-2`: 192.168.1.201:8081 - UP - version: 2.0.0
   - `order-3`: 192.168.1.202:8081 - **STARTING** - version: 2.1.0 (staging)

3. **payment-service** (1 个实例)
   - `payment-1`: 192.168.1.300:8082 - UP - version: 1.5.0

---

## 🧪 快速功能测试

### ✅ 必测功能 (核心)

1. **仪表盘** (`/`)
   - 查看统计卡片: 3 个服务, 6 个实例, 5 个健康实例
   - 查看服务分布饼图
   - 查看实例状态柱状图

2. **服务列表** (`/services`)
   - 查看所有服务
   - 搜索服务 (例如: "user")
   - 展开服务查看实例
   - 刷新数据

3. **实例管理** (`/instances`)
   - 查看所有实例
   - 按状态过滤 (Up / Starting)
   - 按服务过滤
   - 查看实例详情和 metadata

4. **集群管理** (`/cluster`)
   - 查看节点信息 (node1)
   - 查看节点状态 (Healthy)

### 🔧 高级功能 (可选)

5. **路由分组** (`/routing/groups`)
   - 页面加载
   - 查看空状态或现有分组

6. **路由规则** (`/routing/rules`)
   - 页面加载
   - 查看空状态或现有规则

7. **Zone 管理** (`/zone`)
   - 页面加载
   - 查看 Zone 操作

8. **金丝雀发布** (`/canary`)
   - 页面加载
   - 查看金丝雀配置

9. **审计日志** (`/audit`)
   - 页面加载
   - 查看操作日志

---

## 📱 响应式测试

测试不同屏幕尺寸:

1. **桌面** (>1200px) - 侧边栏固定
2. **平板** (768px-1200px) - 侧边栏可折叠
3. **移动** (<768px) - 侧边栏抽屉模式

使用浏览器开发者工具 (F12) → Device Toolbar 切换视图

---

## 🛠 常用命令

### 查看服务状态
```bash
# 后端服务日志
tail -f scripts/.cluster/logs/node1.log

# 前端服务日志
tail -f /tmp/claude-1000/-home-koqizhao-Projects-mydotey-ai-ai-artemis/tasks/b96e597.output
```

### 重新准备测试数据
```bash
./scripts/test-web-console.sh
```

### 手动注册新实例
```bash
curl -X POST http://localhost:8080/api/registry/register.json \
  -H "Content-Type: application/json" \
  -d '{
    "instances": [{
      "region_id": "local",
      "zone_id": "zone1",
      "service_id": "new-service",
      "instance_id": "new-1",
      "ip": "192.168.1.111",
      "port": 9999,
      "url": "http://192.168.1.111:9999",
      "status": "up",
      "metadata": {"test": "true"}
    }]
  }'
```

### 查询所有服务
```bash
curl -s -X POST http://localhost:8080/api/discovery/services.json \
  -H "Content-Type: application/json" \
  -d '{"region_id":"local","zone_id":"zone1"}' | jq '.services[].service_id'
```

---

## 🐛 常见问题排查

### 1. 前端无法连接后端

**症状**: API 请求失败, Network Error

**解决**:
```bash
# 检查后端服务
curl http://localhost:8080/health

# 检查后端日志
tail -f scripts/.cluster/logs/node1.log

# 重启后端
./scripts/cluster.sh restart 1
```

### 2. 数据不显示

**症状**: 页面正常但数据为空

**解决**:
```bash
# 重新准备测试数据
./scripts/test-web-console.sh

# 验证数据
curl -s -X POST http://localhost:8080/api/discovery/services.json \
  -H "Content-Type: application/json" \
  -d '{"region_id":"local","zone_id":"zone1"}' | jq '.'
```

### 3. 前端编译错误

**症状**: Vite 报错, 白屏

**解决**:
```bash
# 进入前端目录
cd artemis-console

# 重新安装依赖
npm install

# 重启开发服务器
npm run dev
```

---

## 📋 详细测试清单

完整的测试清单请参考:
**[docs/web-console/manual-testing-checklist.md](docs/web-console/manual-testing-checklist.md)**

---

## ✅ 测试完成后

测试完成后,请提供以下反馈:

1. **通过的功能**: 哪些功能正常工作
2. **发现的问题**: Bug、UI 问题、性能问题
3. **改进建议**: 功能增强、用户体验优化
4. **整体评价**: 优秀/良好/需改进/不可用

---

**祝测试顺利!** 🎉
