# 配置文件重组报告 - 2026-02-16

## 📋 变更概述

**变更时间**: 2026-02-16
**变更类型**: 项目结构优化 - 配置文件集中化
**影响范围**: 配置文件示例位置和文档引用

---

## 🎯 变更目标

将项目根目录下的配置文件示例移动到 `config/examples/` 目录,实现配置文件的统一管理。

---

## 📦 移动的配置文件

共移动 **3 个配置文件**从根目录到 `config/examples/`:

| # | 文件名 | 数据库 | 用途 | 文件大小 |
|---|--------|--------|------|----------|
| 1 | `artemis-sqlite.toml` | SQLite | 开发/测试环境单节点配置 | 533 B |
| 2 | `artemis-mysql.toml` | MySQL | 生产环境集群配置 | 689 B |
| 3 | `artemis-test-with-db.toml` | SQLite | 集成测试持久化配置 | 620 B |

---

## 📁 目录结构变更

### 变更前
```
ai-artemis/
├── artemis-sqlite.toml              # 配置示例
├── artemis-mysql.toml               # 配置示例
├── artemis-test-with-db.toml        # 配置示例
└── config/
    └── (空目录或其他配置)
```

### 变更后
```
ai-artemis/
└── config/
    └── examples/                    ✨ 新建
        ├── README.md                ✨ 新增
        ├── artemis-sqlite.toml      📦 移动
        ├── artemis-mysql.toml       📦 移动
        └── artemis-test-with-db.toml 📦 移动
```

---

## 📝 文档更新

### 更新的文档数量
- **2 个 Markdown 文档**中的路径引用已更新
- **1 个新文档**创建 (`config/examples/README.md`)

### 主要文档更新

#### 1. 数据库配置文档 (2 个)
| 文档 | 更新数量 | 说明 |
|------|---------|------|
| `docs/DATABASE.md` | 6 处引用 | 所有配置文件路径已更新 |
| `docs/database-configuration-guide.md` | 5 处引用 | 所有配置文件路径已更新 |

#### 2. 路径变更规则
| 原路径 | 新路径 |
|--------|--------|
| `artemis-sqlite.toml` | `config/examples/artemis-sqlite.toml` |
| `artemis-mysql.toml` | `config/examples/artemis-mysql.toml` |
| `artemis-test-with-db.toml` | `config/examples/artemis-test-with-db.toml` |

### 更新示例

**变更前**:
```bash
./artemis server --config artemis-sqlite.toml
```

**变更后**:
```bash
./artemis server --config config/examples/artemis-sqlite.toml
```

---

## ✨ 新增内容

### config/examples/README.md
**创建时间**: 2026-02-16
**文件大小**: ~8 KB
**内容**: 完整的配置文件使用说明文档

**包含内容**:
- 📁 配置文件列表和对比表
- 🚀 三种配置的快速开始指南
  - `artemis-sqlite.toml` - 开发环境详细说明
  - `artemis-mysql.toml` - 生产环境详细说明
  - `artemis-test-with-db.toml` - 测试配置详细说明
- 📊 数据库选择对比 (SQLite vs MySQL)
- ⚙️ 配置文件结构完整说明
- 📝 自定义配置步骤指南
- 🔒 安全建议 (环境变量、文件权限等)
- 🆘 故障排查指南
- 📚 相关文档链接

---

## 📊 配置文件说明

### 1. artemis-sqlite.toml - 开发环境配置

**特点**:
- ✅ SQLite 数据库 (无需外部服务)
- ✅ 单节点模式 (cluster.enabled = false)
- ✅ 友好日志格式 (format = "pretty")
- ✅ 数据持久化到 `artemis.db`

**适用场景**:
- 本地开发
- 快速测试
- 小规模部署

**关键配置**:
```toml
[cluster]
enabled = false

[database]
db_type = "sqlite"
url = "sqlite://artemis.db"
max_connections = 10

[logging]
level = "info"
format = "pretty"
```

---

### 2. artemis-mysql.toml - 生产环境配置

**特点**:
- ✅ MySQL 数据库 (高性能)
- ✅ 集群模式 (cluster.enabled = true)
- ✅ 数据复制 (replication.enabled = true)
- ✅ JSON 日志格式 (便于采集)

**适用场景**:
- 生产环境
- 多节点集群
- 大规模部署

**关键配置**:
```toml
[cluster]
enabled = true
peers = ["http://node-2:8080", "http://node-3:8080"]

[replication]
enabled = true
batch_size = 100
batch_interval_ms = 100

[database]
db_type = "mysql"
url = "mysql://artemis:password@localhost:3306/artemis"
max_connections = 20

[logging]
level = "info"
format = "json"
```

---

### 3. artemis-test-with-db.toml - 测试配置

**特点**:
- ✅ SQLite 数据库 (便于测试)
- ✅ 单节点 (cluster.enabled = false)
- ⚠️ 复制逻辑启用 (用于测试复制功能)
- ✅ 持久化测试数据

**适用场景**:
- 集成测试
- 持久化功能测试
- 复制逻辑测试

**关键配置**:
```toml
[cluster]
enabled = false

[replication]
enabled = true  # 仅用于测试复制逻辑

[database]
url = "sqlite://artemis-test.db"
max_connections = 10
```

---

## 🔍 验证结果

### 文件移动验证
```bash
✅ config/examples/artemis-sqlite.toml 存在
✅ config/examples/artemis-mysql.toml 存在
✅ config/examples/artemis-test-with-db.toml 存在
✅ config/examples/README.md 已创建
✅ 根目录无 artemis-*.toml 文件
```

### 文档引用验证
```bash
✅ docs/DATABASE.md 路径已更新 (6 处)
✅ docs/database-configuration-guide.md 路径已更新 (5 处)
✅ 无遗漏的旧路径引用
```

---

## 📊 影响评估

### 用户使用变更

#### 启动服务器
**变更前**:
```bash
./target/release/artemis server --config artemis-sqlite.toml
```

**变更后**:
```bash
./target/release/artemis server --config config/examples/artemis-sqlite.toml
```

#### 复制配置
**变更前**:
```bash
cp artemis-mysql.toml my-config.toml
```

**变更后**:
```bash
cp config/examples/artemis-mysql.toml config/my-config.toml
```

### 项目收益

1. **目录清晰**
   - 根目录更简洁,仅保留核心文档
   - 配置文件集中在 `config/` 目录

2. **统一管理**
   - 所有配置示例在 `config/examples/`
   - 用户自定义配置推荐放在 `config/`
   - 集群配置示例在 `scripts/examples/`

3. **文档完善**
   - 新增 8KB 的详细配置说明文档
   - 包含快速开始、对比表、故障排查等

4. **易于发现**
   - 新用户通过 `config/examples/README.md` 快速了解
   - 配置文件用途和使用方法一目了然

---

## 🎯 建议的配置文件组织

### config/ 目录结构
```
config/
├── examples/                    # 配置示例 (只读参考)
│   ├── README.md
│   ├── artemis-sqlite.toml
│   ├── artemis-mysql.toml
│   └── artemis-test-with-db.toml
│
├── dev.toml                     # 开发环境配置 (用户创建)
├── staging.toml                 # 预发环境配置 (用户创建)
├── prod-node1.toml              # 生产节点1配置 (用户创建)
├── prod-node2.toml              # 生产节点2配置 (用户创建)
└── prod-node3.toml              # 生产节点3配置 (用户创建)
```

### 推荐工作流

**步骤1: 选择模板**
```bash
# 根据场景选择合适的模板
ls config/examples/
```

**步骤2: 复制配置**
```bash
# 开发环境
cp config/examples/artemis-sqlite.toml config/dev.toml

# 生产环境
cp config/examples/artemis-mysql.toml config/prod-node1.toml
```

**步骤3: 编辑配置**
```bash
# 修改节点ID、数据库连接等
vim config/prod-node1.toml
```

**步骤4: 启动服务**
```bash
./target/release/artemis server --config config/prod-node1.toml
```

---

## 🔒 安全建议

### 1. 添加到 .gitignore
建议在 `.gitignore` 添加:
```gitignore
# User configurations (exclude examples)
config/*.toml
!config/examples/
```

这样可以:
- ✅ 防止提交包含敏感信息的配置文件
- ✅ 保留 `config/examples/` 作为参考
- ✅ 用户自定义配置仅在本地

### 2. 使用环境变量
不要在配置文件中硬编码敏感信息:
```toml
[database]
url = "${DATABASE_URL}"  # 从环境变量读取
```

### 3. 文件权限
限制配置文件权限:
```bash
chmod 600 config/*.toml
chmod 644 config/examples/*.toml
```

---

## ✅ 验收检查清单

- [x] artemis-sqlite.toml 已移动到 config/examples/
- [x] artemis-mysql.toml 已移动到 config/examples/
- [x] artemis-test-with-db.toml 已移动到 config/examples/
- [x] config/examples/README.md 已创建
- [x] docs/DATABASE.md 路径引用已更新
- [x] docs/database-configuration-guide.md 路径引用已更新
- [x] 根目录无残留的 artemis-*.toml 文件
- [x] 无遗漏的旧路径引用

---

## 📚 相关文档

- [config/examples/README.md](../../config/examples/README.md) - 配置文件说明 (新增)
- [docs/DATABASE.md](../DATABASE.md) - 数据库配置指南 (已更新)
- [docs/database-configuration-guide.md](../database-configuration-guide.md) - 数据库配置详细说明 (已更新)
- [scripts/CLUSTER.md](../../scripts/CLUSTER.md) - 集群管理指南
- [scripts/examples/](../../scripts/examples/) - 集群配置示例

---

## 🎉 总结

本次配置文件重组成功完成,实现了以下目标:

1. ✅ **3 个配置文件**从根目录移动到 `config/examples/`
2. ✅ **2 个文档**路径引用全部更新
3. ✅ **1 个新文档** (`config/examples/README.md`) 提供完整的配置说明
4. ✅ **项目结构**更加清晰,配置文件统一管理

**影响范围**: 项目结构优化,不影响代码功能
**向后兼容**: 需要更新启动命令中的配置文件路径
**推荐行动**:
- 更新 `.gitignore` 忽略用户配置,保留示例
- 根据 `config/examples/README.md` 创建自定义配置
- 在生产环境使用环境变量管理敏感信息

---

**执行人**: Claude Sonnet 4.5
**审核人**: koqizhao
**完成时间**: 2026-02-16
**变更状态**: ✅ 已完成

---

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
