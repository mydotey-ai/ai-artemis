#!/bin/bash

# ================================================================
# Artemis 集群管理脚本
# ================================================================
#
# 用途: 在本地一键启动/停止多节点 Artemis 集群,用于开发和测试
#
# 功能:
#   - 启动多节点集群 (默认3节点)
#   - 自动生成节点配置文件
#   - 集群节点状态监控
#   - 日志查看和管理
#   - 优雅停止和清理
#
# 使用示例:
#   ./scripts/cluster.sh start           # 启动3节点集群(默认端口8080+)
#   ./scripts/cluster.sh start 5         # 启动5节点集群
#   ./scripts/cluster.sh start 3 9000    # 启动3节点集群,从端口9000开始
#   ./scripts/cluster.sh status          # 查看集群状态
#   ./scripts/cluster.sh logs 1          # 查看节点1日志
#   ./scripts/cluster.sh stop            # 停止集群
#   ./scripts/cluster.sh clean           # 清理所有文件
#
# 详细文档: CLUSTER.md
#
# ================================================================

set -e

# 配置
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CLUSTER_DIR="${SCRIPT_DIR}/.cluster"
LOGS_DIR="${CLUSTER_DIR}/logs"
PIDS_DIR="${CLUSTER_DIR}/pids"
CONFIG_DIR="${CLUSTER_DIR}/config"

# 默认节点数
DEFAULT_NODE_COUNT=3

# 默认端口配置
DEFAULT_BASE_PORT=8080
DEFAULT_BASE_PEER_PORT=9090

# 数据库配置 (通过环境变量控制)
# DB_TYPE: none (默认), sqlite, mysql
# DB_URL: 自定义数据库连接URL (可选)
# DB_MAX_CONN: 最大连接数 (默认10)
# 注意: SQLite 模式下所有节点共享同一个数据库文件
DB_TYPE=${DB_TYPE:-none}
DB_URL=${DB_URL:-}
DB_MAX_CONN=${DB_MAX_CONN:-10}

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# 日志函数
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_debug() {
    echo -e "${BLUE}[DEBUG]${NC} $1"
}

# 初始化集群目录
init_cluster_dirs() {
    mkdir -p "${LOGS_DIR}"
    mkdir -p "${PIDS_DIR}"
    mkdir -p "${CONFIG_DIR}"

    # 如果使用SQLite,创建数据目录
    if [ "${DB_TYPE}" = "sqlite" ]; then
        mkdir -p "${CLUSTER_DIR}/data"
        log_info "初始化集群目录: ${CLUSTER_DIR} (SQLite数据目录已创建)"
    else
        log_info "初始化集群目录: ${CLUSTER_DIR}"
    fi
}

# 生成节点配置
generate_node_config() {
    local node_id=$1
    local port=$2
    local peer_port=$3
    local peer_nodes=$4

    cat > "${CONFIG_DIR}/node${node_id}.toml" <<EOF
# Artemis Node ${node_id} Configuration

[server]
# 节点ID
node_id = "node${node_id}"

# HTTP API 监听地址
listen_addr = "127.0.0.1:${port}"

# 集群对等节点通信端口
peer_port = ${peer_port}

# 区域和可用区
region = "local"
zone = "zone1"

[cluster]
# 启用集群模式
enabled = true

# 对等节点列表 (其他节点的 HTTP API 地址)
peers = [
${peer_nodes}]

# 数据复制配置
[replication]
# 启用复制
enabled = true

# 复制超时时间(秒)
timeout_secs = 5

# 批量大小
batch_size = 100

# 批处理窗口时间(毫秒)
batch_interval_ms = 100

# 最大重试次数
max_retries = 3

[lease]
# 租约TTL(秒)
ttl_secs = 30

# 租约清理间隔(秒)
cleanup_interval_secs = 60

[cache]
# 启用版本化缓存
enabled = true

# 缓存过期时间(秒)
expiry_secs = 300

[ratelimit]
# 启用限流
enabled = true

# 每秒请求数限制
requests_per_second = 10000

# 突发流量限制
burst_size = 5000

[logging]
# 日志级别: trace, debug, info, warn, error
level = "info"

# 日志格式: json, pretty
format = "pretty"
EOF

    # 添加数据库配置 (如果启用)
    if [ "${DB_TYPE}" != "none" ]; then
        generate_db_config "${node_id}"
    fi

    log_debug "生成节点 ${node_id} 配置: ${CONFIG_DIR}/node${node_id}.toml"
}

# 生成数据库配置
generate_db_config() {
    local node_id=$1
    local db_url="${DB_URL}"

    # 如果未指定URL,使用默认值
    if [ -z "${db_url}" ]; then
        case "${DB_TYPE}" in
            sqlite)
                # SQLite 共享模式: 所有节点使用同一个数据库文件
                db_url="sqlite:${CLUSTER_DIR}/data/shared.db?mode=rwc"
                ;;
            mysql)
                # MySQL默认配置(所有节点共享同一个数据库)
                # 注意: 需要提前创建数据库
                db_url="mysql://artemis:artemis_password@localhost:3306/artemis"
                log_warn "使用默认MySQL配置,请确保数据库已创建并修改密码"
                ;;
            *)
                log_error "不支持的数据库类型: ${DB_TYPE}"
                return 1
                ;;
        esac
    fi

    cat >> "${CONFIG_DIR}/node${node_id}.toml" <<EOF

[database]
# 数据库类型: sqlite, mysql
db_type = "${DB_TYPE}"

# 数据库连接 URL
url = "${db_url}"

# 最大连接数
max_connections = ${DB_MAX_CONN}
EOF

    log_debug "已添加 ${DB_TYPE} 数据库配置到节点 ${node_id}"
}

# 生成所有节点的对等节点列表
# peers 列表应该使用其他节点的 HTTP API 地址,用于集群通信
generate_peer_list() {
    local node_count=$1
    local base_port=$2
    local current_node=$3

    local peers=""
    for i in $(seq 1 ${node_count}); do
        if [ $i -ne ${current_node} ]; then
            peers="${peers}    \"127.0.0.1:$((base_port + i - 1))\",\n"
        fi
    done

    # 移除最后一个逗号
    echo -e "${peers}" | sed '$ s/,$//'
}

# 启动单个节点
start_node() {
    local node_id=$1
    local port=$2
    local config_file="${CONFIG_DIR}/node${node_id}.toml"
    local pid_file="${PIDS_DIR}/node${node_id}.pid"
    local log_file="${LOGS_DIR}/node${node_id}.log"

    # 检查节点是否已启动
    if [ -f "${pid_file}" ]; then
        local pid=$(cat "${pid_file}")
        if kill -0 "${pid}" 2>/dev/null; then
            log_warn "节点 ${node_id} 已在运行 (PID: ${pid})"
            return 0
        fi
    fi

    log_info "启动节点 ${node_id} (端口: ${port})..."

    # 使用已编译的二进制启动 Artemis (避免重复编译)
    cd "${SCRIPT_DIR}"

    # 检查二进制是否已存在
    local artemis_binary="${SCRIPT_DIR}/../target/release/artemis"
    if [ ! -f "${artemis_binary}" ]; then
        log_warn "二进制文件不存在，正在编译..."
        cd "${SCRIPT_DIR}/.."
        cargo build --release --bin artemis
        cd "${SCRIPT_DIR}"
    fi

    # 使用配置文件启动服务器
    RUST_LOG=info "${artemis_binary}" server --config "${config_file}" \
        > "${log_file}" 2>&1 &

    local pid=$!
    echo "${pid}" > "${pid_file}"

    # 等待节点启动
    sleep 2

    # 验证进程是否存在
    if kill -0 "${pid}" 2>/dev/null; then
        log_info "节点 ${node_id} 已启动 (PID: ${pid}, 端口: ${port})"
    else
        log_error "节点 ${node_id} 启动失败,请查看日志: ${log_file}"
        rm -f "${pid_file}"
        return 1
    fi
}

# 停止单个节点
stop_node() {
    local node_id=$1
    local pid_file="${PIDS_DIR}/node${node_id}.pid"

    if [ ! -f "${pid_file}" ]; then
        log_warn "节点 ${node_id} 未运行"
        return 0
    fi

    local pid=$(cat "${pid_file}")

    if ! kill -0 "${pid}" 2>/dev/null; then
        log_warn "节点 ${node_id} 进程不存在 (PID: ${pid})"
        rm -f "${pid_file}"
        return 0
    fi

    log_info "停止节点 ${node_id} (PID: ${pid})..."

    # 发送 SIGTERM 信号
    kill "${pid}"

    # 等待进程退出
    local timeout=10
    while kill -0 "${pid}" 2>/dev/null && [ ${timeout} -gt 0 ]; do
        sleep 1
        timeout=$((timeout - 1))
    done

    # 如果还未退出,强制终止
    if kill -0 "${pid}" 2>/dev/null; then
        log_warn "节点 ${node_id} 未响应 SIGTERM,发送 SIGKILL..."
        kill -9 "${pid}"
        sleep 1
    fi

    rm -f "${pid_file}"
    log_info "节点 ${node_id} 已停止"
}

# 获取节点状态
get_node_status() {
    local node_id=$1
    local port=$2
    local pid_file="${PIDS_DIR}/node${node_id}.pid"

    if [ ! -f "${pid_file}" ]; then
        echo -e "节点 ${node_id}: ${RED}未运行${NC}"
        return 1
    fi

    local pid=$(cat "${pid_file}")

    if ! kill -0 "${pid}" 2>/dev/null; then
        echo -e "节点 ${node_id}: ${RED}已停止${NC} (PID 文件存在但进程不存在)"
        return 1
    fi

    # 尝试访问健康检查端点
    if curl -s -f "http://127.0.0.1:${port}/health" > /dev/null 2>&1; then
        echo -e "节点 ${node_id}: ${GREEN}运行中${NC} (PID: ${pid}, 端口: ${port})"
    else
        echo -e "节点 ${node_id}: ${YELLOW}启动中${NC} (PID: ${pid}, 端口: ${port})"
    fi
}

# 启动集群
start_cluster() {
    local node_count=${1:-${DEFAULT_NODE_COUNT}}
    local base_port=${2:-${DEFAULT_BASE_PORT}}
    local base_peer_port=${3:-${DEFAULT_BASE_PEER_PORT}}

    # 显示数据库配置信息
    if [ "${DB_TYPE}" != "none" ]; then
        log_info "启动 ${node_count} 节点 Artemis 集群 (数据库: ${DB_TYPE})..."
        if [ "${DB_TYPE}" = "sqlite" ]; then
            log_warn "注意: SQLite 模式下所有节点共享数据库,并发写入性能有限"
            log_warn "生产环境建议使用 MySQL 数据库"
        fi
    else
        log_info "启动 ${node_count} 节点 Artemis 集群 (纯内存模式)..."
    fi

    init_cluster_dirs

    # 生成配置文件
    for i in $(seq 1 ${node_count}); do
        local port=$((base_port + i - 1))
        local peer_port=$((base_peer_port + i - 1))
        local peer_nodes=$(generate_peer_list ${node_count} ${base_port} ${i})
        generate_node_config ${i} ${port} ${peer_port} "${peer_nodes}"
    done

    # 启动所有节点
    for i in $(seq 1 ${node_count}); do
        local port=$((base_port + i - 1))
        start_node ${i} ${port}
    done

    log_info "集群启动完成!"
    log_info "节点端口范围: ${base_port}-$((base_port + node_count - 1))"
    log_info ""
    log_info "查看节点日志: tail -f ${LOGS_DIR}/node*.log"
    log_info "查看集群状态: $0 status"
}

# 停止集群
stop_cluster() {
    log_info "停止 Artemis 集群..."

    # 查找所有 PID 文件
    if [ ! -d "${PIDS_DIR}" ]; then
        log_warn "集群未运行"
        return 0
    fi

    local pid_files=$(find "${PIDS_DIR}" -name "*.pid" 2>/dev/null)

    if [ -z "${pid_files}" ]; then
        log_warn "未找到运行中的节点"
        return 0
    fi

    # 停止所有节点
    for pid_file in ${pid_files}; do
        local node_id=$(basename "${pid_file}" .pid | sed 's/node//')
        stop_node ${node_id}
    done

    log_info "集群已停止"
}

# 重启集群
restart_cluster() {
    log_info "重启 Artemis 集群..."
    stop_cluster
    sleep 2
    start_cluster "$@"
}

# 查看集群状态
status_cluster() {
    local base_port=${1:-${DEFAULT_BASE_PORT}}

    log_info "Artemis 集群状态:"
    echo ""

    if [ ! -d "${PIDS_DIR}" ]; then
        log_warn "集群未初始化"
        return 0
    fi

    local pid_files=$(find "${PIDS_DIR}" -name "*.pid" 2>/dev/null | sort)

    if [ -z "${pid_files}" ]; then
        log_warn "未找到运行中的节点"
        return 0
    fi

    for pid_file in ${pid_files}; do
        local node_id=$(basename "${pid_file}" .pid | sed 's/node//')
        local port=$((base_port + node_id - 1))
        get_node_status ${node_id} ${port}
    done
}

# 查看节点日志
logs_cluster() {
    local node_id=${1:-""}

    if [ -z "${node_id}" ]; then
        # 查看所有节点日志
        if [ -d "${LOGS_DIR}" ]; then
            tail -f "${LOGS_DIR}"/*.log
        else
            log_error "日志目录不存在: ${LOGS_DIR}"
            return 1
        fi
    else
        # 查看指定节点日志
        local log_file="${LOGS_DIR}/node${node_id}.log"
        if [ -f "${log_file}" ]; then
            tail -f "${log_file}"
        else
            log_error "日志文件不存在: ${log_file}"
            return 1
        fi
    fi
}

# 清理集群文件
clean_cluster() {
    log_info "清理集群文件..."

    # 确保集群已停止
    stop_cluster

    # 删除集群目录
    if [ -d "${CLUSTER_DIR}" ]; then
        rm -rf "${CLUSTER_DIR}"
        log_info "集群文件已清理"
    else
        log_warn "集群目录不存在"
    fi
}

# 显示帮助信息
show_help() {
    cat <<EOF
Artemis 集群管理脚本

用法: $0 <命令> [选项]

命令:
    start [节点数] [基础端口]
        启动集群
        默认: 3 节点, 基础端口 8080
        节点将使用连续端口: 基础端口, 基础端口+1, ...
        示例: $0 start 5 9000  # 启动5节点,端口9000-9004

    stop
        停止集群

    restart [节点数] [基础端口]
        重启集群

    status [基础端口]
        查看集群状态
        默认基础端口: 8080

    logs [节点ID]
        查看日志
        不指定节点ID则查看所有节点日志
        示例: $0 logs 1

    clean
        停止集群并清理所有文件

    help
        显示此帮助信息

示例:
    # 启动 3 节点集群 (默认,端口 8080-8082)
    $0 start

    # 启动 5 节点集群,自定义端口 (9000-9004)
    $0 start 5 9000

    # 查看集群状态
    $0 status

    # 查看所有节点日志
    $0 logs

    # 查看节点 1 的日志
    $0 logs 1

    # 停止集群
    $0 stop

    # 清理所有文件
    $0 clean

集群配置说明:
    - 每个节点使用独立的 HTTP API 端口 (基础端口 + 节点编号 - 1)
    - 每个节点使用独立的对等通信端口 (默认 9090 + 节点编号 - 1)
    - 集群节点通过 HTTP API 端口相互通信
    - 所有配置文件、日志和 PID 文件存储在 .cluster 目录

数据库配置 (通过环境变量):
    DB_TYPE         数据库类型: none (默认), sqlite, mysql
    DB_URL          自定义数据库连接URL (可选,有默认值)
    DB_MAX_CONN     最大连接数 (默认: 10)

    数据库使用示例:
    # SQLite 模式 (所有节点共享 shared.db)
    DB_TYPE=sqlite $0 start

    # MySQL 集群模式
    DB_TYPE=mysql DB_URL="mysql://user:pass@host:3306/artemis" $0 start

    注意:
    - SQLite 模式下所有节点共享同一个数据库文件
    - SQLite 并发写入性能有限,生产环境建议使用 MySQL

    ⚠️  重要说明:
    - cluster.sh 主要用于开发环境,默认推荐纯内存模式(无数据库)
    - 数据库配置功能可生成配置文件,但可能需要特殊编译才能运行
    - 生产环境请使用 config/production-cluster-node*.toml 配置文件
    - 详细文档: docs/database-configuration-guide.md

集群文件位置:
    配置: ${CONFIG_DIR}
    日志: ${LOGS_DIR}
    PID: ${PIDS_DIR}
    数据: ${CLUSTER_DIR}/data (仅SQLite)
EOF
}

# 主函数
main() {
    local command=${1:-"help"}
    shift || true

    case "${command}" in
        start)
            start_cluster "$@"
            ;;
        stop)
            stop_cluster
            ;;
        restart)
            restart_cluster "$@"
            ;;
        status)
            status_cluster "$@"
            ;;
        logs)
            logs_cluster "$@"
            ;;
        clean)
            clean_cluster
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            log_error "未知命令: ${command}"
            echo ""
            show_help
            exit 1
            ;;
    esac
}

# 执行主函数
main "$@"
