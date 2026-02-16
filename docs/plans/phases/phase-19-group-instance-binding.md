# Phase 19: 分组实例绑定

**优先级**: P1 (高优先级)
**预估工时**: 5 天
**状态**: ✅ 已完成 (2026-02-15)

---

## 📋 功能概述

实现手动绑定实例到分组的功能，支持手动和自动两种绑定模式。

**Java 版本功能**:
- 支持手动添加实例到分组 (`insert-group-instances.json`)
- 支持从分组移除实例 (`delete-group-instances.json`)
- 支持批量添加服务实例 (`insert-service-instances.json`)

**Rust 实现**:
- 在 Phase 13 基础上扩展，添加手动绑定功能
- 实现 GroupInstanceDao 持久化层
- 支持手动/自动绑定模式切换

---

## 🎯 实现目标

### 3 个 API 端点

1. **手动添加实例** - `POST /api/routing/groups/{group_key}/instances`
2. **移除实例** - `DELETE /api/routing/groups/{group_key}/instances/{instance_id}`
3. **批量添加服务实例** - `POST /api/routing/groups/{group_key}/service-instances`

---

## ✅ 实现状态

- ✅ 数据模型 (GroupInstanceBinding)
- ✅ DAO 层持久化 (GroupInstanceDao)
- ✅ 手动/自动绑定支持
- ✅ 批量添加服务实例
- ✅ 3 个 API 端点
- ✅ 9 步集成测试 (test-group-instance-binding.sh)

---

## 📖 相关文档

- **基础功能**: Phase 13 分组路由
- **实现代码**: `artemis-management/src/group_instance_dao.rs`
- **测试脚本**: `scripts/test-group-instance-binding.sh`
- **原始设计**: `docs/archive/phase-19-22-gap-fixing-plan.md`

---

**完成时间**: 2026-02-15
**集成测试**: ✅ 9 步测试全部通过
