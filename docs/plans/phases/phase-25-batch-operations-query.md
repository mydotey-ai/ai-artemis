# Phase 25: 批量操作查询 API

**优先级**: P1 (重要)
**预估工时**: 1 天
**状态**: ✅ 已完成 (2026-02-15)

---

## 📋 功能概述

查询所有实例和服务器的操作历史，支持 POST/GET 双模式。

---

## 🎯 实现目标

### 4 个查询 API

1. **查询所有实例操作 (POST)** - `POST /api/management/instance/get-all-operations.json`
2. **查询所有实例操作 (GET)** - `GET /api/management/instance/get-all-operations.json`
3. **查询所有服务器操作 (POST)** - `POST /api/management/server/get-all-operations.json`
4. **查询所有服务器操作 (GET)** - `GET /api/management/server/get-all-operations.json`

**查询参数**:
- `region_id` - 按 Region 过滤

---

## ✅ 实现状态

- ✅ POST/GET 双模式支持
- ✅ Region 过滤
- ✅ 统一响应格式 (ResponseStatus)
- ✅ 11 步集成测试 (test-all-operations.sh)

---

## 📖 相关文档

- **基础功能**: Phase 12 实例管理
- **实现代码**: `artemis-server/src/handlers/management.rs`
- **测试脚本**: `scripts/test-all-operations.sh`

---

**完成时间**: 2026-02-15
**集成测试**: ✅ 11 步测试全部通过
