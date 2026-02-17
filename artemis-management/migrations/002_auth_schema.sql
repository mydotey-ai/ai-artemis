-- Artemis Authentication System Schema
-- 3张表用于用户认证和会话管理

-- 1. 用户表
CREATE TABLE IF NOT EXISTS auth_users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL UNIQUE,
    username TEXT NOT NULL UNIQUE,
    email TEXT,
    password_hash TEXT NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('admin', 'operator', 'viewer')),
    status TEXT NOT NULL CHECK(status IN ('active', 'inactive')),
    description TEXT,
    created_at BIGINT NOT NULL,
    updated_at BIGINT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_users_username ON auth_users(username);
CREATE INDEX IF NOT EXISTS idx_users_role ON auth_users(role);
CREATE INDEX IF NOT EXISTS idx_users_status ON auth_users(status);

-- 2. 会话表
CREATE TABLE IF NOT EXISTS auth_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL UNIQUE,
    user_id TEXT NOT NULL,
    token TEXT NOT NULL UNIQUE,
    ip_address TEXT,
    user_agent TEXT,
    created_at BIGINT NOT NULL,
    expires_at BIGINT NOT NULL,
    last_activity BIGINT NOT NULL,
    FOREIGN KEY(user_id) REFERENCES auth_users(user_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_sessions_user ON auth_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_token ON auth_sessions(token);
CREATE INDEX IF NOT EXISTS idx_sessions_expires ON auth_sessions(expires_at);

-- 3. 登录历史表
CREATE TABLE IF NOT EXISTS auth_login_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id TEXT NOT NULL,
    login_time BIGINT NOT NULL,
    ip_address TEXT NOT NULL,
    user_agent TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('success', 'failed')),
    FOREIGN KEY(user_id) REFERENCES auth_users(user_id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_login_history_user ON auth_login_history(user_id);
CREATE INDEX IF NOT EXISTS idx_login_history_time ON auth_login_history(login_time);
CREATE INDEX IF NOT EXISTS idx_login_history_status ON auth_login_history(status);
