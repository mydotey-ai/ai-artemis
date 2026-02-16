# 项目文档组织规范

## 📁 目录结构

```
ai-artemis/
├── README.md                           # 项目首页 - 快速开始和 API 使用
├── CLAUDE.md                           # 项目完成总结
├── CLUSTER.md                          # 集群管理指南
│
├── .claude/                            # Claude 助手规则（版本化）
│   └── rules/                          # 项目规则
│       └── doc.md                       # 文档组织规范
│
├── docs/                               # 文档中心
│   ├── README.md                       # 文档导航索引
│   ├── artemis-rust-rewrite-specification.md  # 产品规格说明
│   ├── deployment.md                   # 部署指南
│   │
│   ├── plans/                          # 设计和计划文档（版本化）
│   │   ├── README.md                   # 计划文档索引
│   │   ├── design.md                   # 架构设计
│   │   ├── implementation-roadmap.md   # 实施路线图
│   │   └── phases/                     # Phase 详细计划
│   │
│   └── testing/                        # 测试文档（版本化）
│       ├── README.md                   # 测试文档导航
│       ├── test-status.md              # 测试状态报告
│       ├── test-strategy.md            # 测试策略和最佳实践
│       └── CHANGELOG.md                # 文档变更历史
│
├── docs/reports/                       # 进度报告（不版本化）
│   └── ...                             # 临时报告，定期清理
│
└── docs/archive/                       # 归档文档（不版本化）
    └── ...                             # 历史文档，仅本地参考
```

## 📂 版本化管理说明

### ✅ 版本化目录（纳入 git 管理）

| 目录 | 用途 | 示例 |
|------|------|------|
| `.claude/` | Claude 助手规则 | rules/doc.md |
| `docs/plans/` | 设计和计划文档 | design.md, implementation-roadmap.md |
| `docs/testing/` | 测试文档 | test-status.md, test-strategy.md |

### ❌ 不版本化目录（仅本地保留）

| 目录 | 用途 | 说明 |
|------|------|------|
| `docs/reports/` | 进度报告 | 临时报告，定期清理 |
| `docs/archive/` | 归档文档 | 历史文档，仅本地参考 |

> **注意**: 不版本化的目录已添加到 `.gitignore`，不会被提交到版本库。

## 📐 文档命名规范

### 目录命名
- **全小写** + **连字符分隔**: `plans/`, `reports/`, `archive/`
- **复数形式**: 包含多个文档的目录用复数 (`phases/`, `features/`, `performance/`)

### 文件命名
- **全小写** + **连字符分隔**: `design.md`, `implementation-roadmap.md`
- **Phase 文档**: 统一格式 `phase-XX-name.md` (XX 为两位数字,01-25)
- **避免日期后缀**: 使用描述性名称,不在文件名中包含日期
- **归档文档**: 历史文档移至 `archive/` 目录,保持原意义的简化命名

## 📂 文档分类说明

### 📐 plans/ - 设计和计划文档
**用途**: 架构设计、技术选型、实施计划
**管理**: ✅ 版本化
**包含**:
- `design.md` - 系统架构、模块结构、数据模型
- `implementation-roadmap.md` - 分阶段实施计划和优先级
- `phases/` - Phase 的详细任务计划

### 🧪 testing/ - 测试文档
**用途**: 测试状态、策略和最佳实践
**管理**: ✅ 版本化
**包含**:
- `test-status.md` - 当前测试统计
- `test-strategy.md` - 测试方法论
- `CHANGELOG.md` - 文档变更历史

### 📊 reports/ - 进度报告
**用途**: 项目状态、功能实现、性能测试报告
**管理**: ❌ 不版本化
**包含**:
- `features/` - 功能实现报告
- `performance/` - 性能测试报告
- `archive/` - 历史归档

### 📁 archive/ - 归档文档
**用途**: 保存历史记录和阶段性总结
**管理**: ❌ 不版本化
**包含**:
- `session-reports/` - Session 报告和整理文档
- `test-reports/` - 测试报告和总结

## 📋 维护规则

### 1. 单一信息源 (Single Source of Truth)
- 同一信息只在一处维护
- 其他位置通过引用链接
- 避免内容重复和不一致

### 2. 清晰的层次结构
- 三级分类: 主题目录 → 子分类 → 具体文档
- 每级目录都有 README.md 索引
- 文档路径直观反映内容分类

### 3. 版本和状态标记
- 使用状态标识: ✅ 最新 | ⚠️ 待更新 | 📁 历史文档
- 每个文档标注最后更新时间
- 重大变更在文档开头说明

### 4. 冗余文档合并
- 定期检查并合并重复内容
- 保留最全面的版本
- 更新所有相关引用

## 🔗 相关文档

- **项目文档**: `docs/README.md`
- **架构设计**: `docs/plans/design.md`
- **实施路线图**: `docs/plans/implementation-roadmap.md`
- **测试文档**: `docs/testing/README.md`
