-- Artemis 数据持久化 Schema
-- 12张表用于管理配置持久化存储

-- 1. 实例操作表
CREATE TABLE IF NOT EXISTS instance_operation (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    region_id TEXT NOT NULL,
    service_id TEXT NOT NULL,
    instance_id TEXT NOT NULL,
    ip TEXT NOT NULL,
    port INTEGER NOT NULL,
    zone_id TEXT,
    operation TEXT NOT NULL CHECK(operation IN ('pullin', 'pullout')),
    operator_id TEXT NOT NULL,
    operation_time BIGINT NOT NULL,
    operation_complete BOOLEAN NOT NULL DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(region_id, service_id, instance_id)
);

CREATE INDEX IF NOT EXISTS idx_instance_op_service ON instance_operation(service_id);
CREATE INDEX IF NOT EXISTS idx_instance_op_server ON instance_operation(ip, region_id);

-- 2. 服务器操作表
CREATE TABLE IF NOT EXISTS server_operation (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    server_id TEXT NOT NULL,
    region_id TEXT NOT NULL,
    operation TEXT NOT NULL CHECK(operation IN ('pullin', 'pullout')),
    operator_id TEXT NOT NULL,
    operation_time BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(server_id, region_id)
);

CREATE INDEX IF NOT EXISTS idx_server_op_region ON server_operation(region_id);

-- 3. 服务分组表
CREATE TABLE IF NOT EXISTS service_group (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id TEXT NOT NULL UNIQUE,
    group_name TEXT NOT NULL,
    group_type TEXT NOT NULL CHECK(group_type IN ('physical', 'logical')),
    service_id TEXT,
    region_id TEXT,
    zone_id TEXT,
    description TEXT,
    metadata TEXT, -- JSON
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_group_service ON service_group(service_id);
CREATE INDEX IF NOT EXISTS idx_group_type ON service_group(group_type);

-- 4. 分组标签表
CREATE TABLE IF NOT EXISTS service_group_tag (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id TEXT NOT NULL,
    tag_key TEXT NOT NULL,
    tag_value TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(group_id, tag_key),
    FOREIGN KEY(group_id) REFERENCES service_group(group_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_group_tag_group ON service_group_tag(group_id);
CREATE INDEX IF NOT EXISTS idx_group_tag_key ON service_group_tag(tag_key);

-- 5. 路由规则表
CREATE TABLE IF NOT EXISTS service_route_rule (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    route_rule_id INTEGER,
    route_id TEXT NOT NULL UNIQUE,
    service_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    status TEXT NOT NULL DEFAULT 'inactive' CHECK(status IN ('active', 'inactive')),
    strategy TEXT NOT NULL CHECK(strategy IN ('weighted-round-robin', 'close-by-visit')),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_route_rule_service ON service_route_rule(service_id);
CREATE INDEX IF NOT EXISTS idx_route_rule_status ON service_route_rule(status);

-- 6. 路由规则分组关联表
CREATE TABLE IF NOT EXISTS service_route_rule_group (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    rule_id TEXT NOT NULL,
    group_id TEXT NOT NULL,
    weight INTEGER NOT NULL DEFAULT 100 CHECK(weight >= 0 AND weight <= 100),
    priority INTEGER NOT NULL DEFAULT 0,
    region_id TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(rule_id, group_id),
    FOREIGN KEY(rule_id) REFERENCES service_route_rule(route_id) ON DELETE CASCADE,
    FOREIGN KEY(group_id) REFERENCES service_group(group_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_rule_group_rule ON service_route_rule_group(rule_id);
CREATE INDEX IF NOT EXISTS idx_rule_group_group ON service_route_rule_group(group_id);

-- 7. Zone 操作表
CREATE TABLE IF NOT EXISTS zone_operation (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    zone_id TEXT NOT NULL,
    region_id TEXT NOT NULL,
    operation TEXT NOT NULL CHECK(operation IN ('pullin', 'pullout')),
    operator_id TEXT NOT NULL,
    operation_time BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(zone_id, region_id)
);

CREATE INDEX IF NOT EXISTS idx_zone_op_zone ON zone_operation(zone_id);
CREATE INDEX IF NOT EXISTS idx_zone_op_region ON zone_operation(region_id);

-- 8. 金丝雀配置表
CREATE TABLE IF NOT EXISTS canary_config (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id TEXT NOT NULL UNIQUE,
    ip_whitelist TEXT NOT NULL, -- JSON array
    enabled BOOLEAN NOT NULL DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_canary_service ON canary_config(service_id);
CREATE INDEX IF NOT EXISTS idx_canary_enabled ON canary_config(enabled);

-- 9. 操作审计日志表
CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    log_id BIGINT NOT NULL UNIQUE,
    operation_type TEXT NOT NULL, -- instance_operation, server_operation, zone_operation, etc.
    target_id TEXT NOT NULL,
    operation TEXT NOT NULL,
    operator_id TEXT NOT NULL,
    operation_time BIGINT NOT NULL,
    details TEXT, -- JSON
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_audit_type ON audit_log(operation_type);
CREATE INDEX IF NOT EXISTS idx_audit_target ON audit_log(target_id);
CREATE INDEX IF NOT EXISTS idx_audit_operator ON audit_log(operator_id);
CREATE INDEX IF NOT EXISTS idx_audit_time ON audit_log(operation_time);

-- 10. 分组实例关联表 (用于逻辑分组)
CREATE TABLE IF NOT EXISTS service_group_instance (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_id TEXT NOT NULL,
    region_id TEXT NOT NULL,
    zone_id TEXT NOT NULL,
    service_id TEXT NOT NULL,
    instance_id TEXT NOT NULL,
    ip TEXT,
    port INTEGER,
    binding_type TEXT NOT NULL DEFAULT 'auto' CHECK(binding_type IN ('manual', 'auto')),
    operator_id TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(group_id, instance_id, region_id, zone_id),
    FOREIGN KEY(group_id) REFERENCES service_group(group_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_group_inst_group ON service_group_instance(group_id);
CREATE INDEX IF NOT EXISTS idx_group_inst_service ON service_group_instance(service_id);
CREATE INDEX IF NOT EXISTS idx_group_inst_binding ON service_group_instance(binding_type);

-- 11. 配置版本表 (用于配置变更追踪)
CREATE TABLE IF NOT EXISTS config_version (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    config_type TEXT NOT NULL, -- route_rule, service_group, zone_operation, etc.
    config_id TEXT NOT NULL,
    version INTEGER NOT NULL,
    content TEXT NOT NULL, -- JSON
    operator_id TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(config_type, config_id, version)
);

CREATE INDEX IF NOT EXISTS idx_config_version_type ON config_version(config_type, config_id);

-- 12. 实例操作日志表 (历史记录)
CREATE TABLE IF NOT EXISTS instance_operation_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    region_id TEXT NOT NULL,
    service_id TEXT NOT NULL,
    instance_id TEXT NOT NULL,
    ip TEXT NOT NULL,
    port INTEGER NOT NULL,
    zone_id TEXT,
    operation TEXT NOT NULL,
    operator_id TEXT NOT NULL,
    operation_time BIGINT NOT NULL,
    operation_complete BOOLEAN NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_instance_log_time ON instance_operation_log(created_at);
CREATE INDEX IF NOT EXISTS idx_instance_log_operator ON instance_operation_log(operator_id);
CREATE INDEX IF NOT EXISTS idx_instance_log_service ON instance_operation_log(service_id);
