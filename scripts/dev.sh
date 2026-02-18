#!/bin/bash

# ================================================================
# Artemis 开发环境一键启动脚本
# ================================================================
#
# 用途: 在开发机上一键启动前后端服务
#
# 功能:
#   - 启动后端集群 (基于 cluster.sh)
#   - 启动前端开发服务器 (artemis-console)
#   - 自动打开浏览器
#   - 统一日志管理
#   - 一键停止所有服务
#
# 使用示例:
#   ./scripts/dev.sh start           # 启动开发环境 (1节点后端 + 前端)
#   ./scripts/dev.sh start 3         # 启动3节点后端集群 + 前端
#   ./scripts/dev.sh status          # 查看服务状态
#   ./scripts/dev.sh logs            # 查看所有日志
#   ./scripts/dev.sh logs backend    # 只看后端日志
#   ./scripts/dev.sh logs frontend   # 只看前端日志
#   ./scripts/dev.sh stop            # 停止所有服务
#   ./scripts/dev.sh restart         # 重启所有服务
#
# ================================================================

set -e

# 配置
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
DEV_DIR="${SCRIPT_DIR}/.dev"
FRONTEND_DIR="${PROJECT_ROOT}/artemis-console"
CLUSTER_SCRIPT="${SCRIPT_DIR}/cluster.sh"

# 默认配置
DEFAULT_NODE_COUNT=3
DEFAULT_BACKEND_PORT=8080
DEFAULT_FRONTEND_PORT=5173

# 数据库配置 (默认使用 SQLite)
DB_TYPE=${DB_TYPE:-sqlite}

# PID 文件
FRONTEND_PID_FILE="${DEV_DIR}/frontend.pid"

# 颜色输出
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
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

log_section() {
    echo -e "\n${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}  $1${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
}

# 初始化开发目录
init_dev_dirs() {
    mkdir -p "${DEV_DIR}"
    log_debug "初始化开发目录: ${DEV_DIR}"
}

# 检查前端依赖
check_frontend_deps() {
    if [ ! -d "${FRONTEND_DIR}/node_modules" ]; then
        log_warn "前端依赖未安装，正在安装..."
        cd "${FRONTEND_DIR}"
        npm install
        cd "${PROJECT_ROOT}"
        log_info "前端依赖安装完成"
    fi
}

# 启动后端集群
start_backend() {
    local node_count=${1:-${DEFAULT_NODE_COUNT}}
    local base_port=${2:-${DEFAULT_BACKEND_PORT}}

    log_section "启动后端服务"

    # 检查 cluster.sh 是否存在
    if [ ! -f "${CLUSTER_SCRIPT}" ]; then
        log_error "cluster.sh 脚本不存在: ${CLUSTER_SCRIPT}"
        return 1
    fi

    # 先编译后端（避免启动时才编译导致超时）
    log_info "正在编译 Rust 后端 (这可能需要几分钟)..."
    cd "${PROJECT_ROOT}"
    if ! cargo build --release --bin artemis; then
        log_error "Rust 编译失败"
        cd "${SCRIPT_DIR}"
        return 1
    fi
    cd "${SCRIPT_DIR}"

    # 启动集群
    log_info "启动 ${node_count} 节点后端集群 (端口: ${base_port}, 数据库: ${DB_TYPE})..."
    DB_TYPE="${DB_TYPE}" "${CLUSTER_SCRIPT}" start "${node_count}" "${base_port}"

    # 等待后端启动
    log_info "等待后端服务就绪..."
    local max_attempts=120
    local attempt=0

    while [ ${attempt} -lt ${max_attempts} ]; do
        if curl -s -f "http://127.0.0.1:${base_port}/health" > /dev/null 2>&1; then
            log_info "后端服务已就绪!"
            return 0
        fi
        attempt=$((attempt + 1))
        if [ $((attempt % 10)) -eq 0 ]; then
            log_info "仍在等待... (${attempt}/${max_attempts})"
        fi
        sleep 1
    done

    log_error "后端服务启动超时 (${max_attempts}秒)"
    log_error "请查看日志: ${SCRIPT_DIR}/.cluster/logs/"
    return 1
}

# 启动前端
start_frontend() {
    log_section "启动前端服务"

    # 检查前端目录
    if [ ! -d "${FRONTEND_DIR}" ]; then
        log_error "前端目录不存在: ${FRONTEND_DIR}"
        return 1
    fi

    # 检查依赖
    check_frontend_deps

    # 检查是否已启动
    if [ -f "${FRONTEND_PID_FILE}" ]; then
        local pid=$(cat "${FRONTEND_PID_FILE}")
        if kill -0 "${pid}" 2>/dev/null; then
            log_warn "前端服务已在运行 (PID: ${pid})"
            return 0
        fi
    fi

    log_info "启动前端开发服务器..."
    cd "${FRONTEND_DIR}"

    # 启动前端 (后台运行)
    npm run dev > "${DEV_DIR}/frontend.log" 2>&1 &
    local pid=$!
    echo "${pid}" > "${FRONTEND_PID_FILE}"

    cd "${PROJECT_ROOT}"

    # 等待前端启动
    log_info "等待前端服务就绪..."
    local max_attempts=30
    local attempt=0

    while [ ${attempt} -lt ${max_attempts} ]; do
        if curl -s -f "http://127.0.0.1:${DEFAULT_FRONTEND_PORT}" > /dev/null 2>&1; then
            log_info "前端服务已就绪 (PID: ${pid})!"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep 1
    done

    log_error "前端服务启动超时"
    return 1
}

# 停止后端
stop_backend() {
    log_section "停止后端服务"
    "${CLUSTER_SCRIPT}" stop
}

# 停止前端
stop_frontend() {
    log_section "停止前端服务"

    if [ ! -f "${FRONTEND_PID_FILE}" ]; then
        log_warn "前端服务未运行"
        return 0
    fi

    local pid=$(cat "${FRONTEND_PID_FILE}")

    if ! kill -0 "${pid}" 2>/dev/null; then
        log_warn "前端服务进程不存在 (PID: ${pid})"
        rm -f "${FRONTEND_PID_FILE}"
        return 0
    fi

    log_info "停止前端服务 (PID: ${pid})..."

    # 发送 SIGTERM
    kill "${pid}"

    # 等待进程退出
    local timeout=10
    while kill -0 "${pid}" 2>/dev/null && [ ${timeout} -gt 0 ]; do
        sleep 1
        timeout=$((timeout - 1))
    done

    # 如果还未退出，强制终止
    if kill -0 "${pid}" 2>/dev/null; then
        log_warn "前端服务未响应 SIGTERM，发送 SIGKILL..."
        kill -9 "${pid}"
        sleep 1
    fi

    rm -f "${FRONTEND_PID_FILE}"
    log_info "前端服务已停止"
}

# 启动开发环境
start_dev() {
    local node_count=${1:-${DEFAULT_NODE_COUNT}}
    local backend_port=${2:-${DEFAULT_BACKEND_PORT}}

    log_section "启动 Artemis 开发环境"

    init_dev_dirs

    # 启动后端
    if ! start_backend "${node_count}" "${backend_port}"; then
        log_error "后端启动失败"
        return 1
    fi

    # 启动前端
    if ! start_frontend; then
        log_error "前端启动失败，停止后端服务..."
        stop_backend
        return 1
    fi

    # 显示访问信息
    log_section "开发环境已启动"

    echo -e "${GREEN}✓ 后端集群:${NC} ${node_count} 节点 (端口: ${backend_port}-$((backend_port + node_count - 1)))"
    echo -e "${GREEN}✓ 数据库:${NC} ${DB_TYPE}"
    echo -e "${GREEN}✓ 前端服务:${NC} http://127.0.0.1:${DEFAULT_FRONTEND_PORT}"
    echo -e ""
    echo -e "快速访问:"
    echo -e "  ${CYAN}Web 控制台:${NC} http://127.0.0.1:${DEFAULT_FRONTEND_PORT}"
    echo -e "  ${CYAN}后端 API:${NC}   http://127.0.0.1:${backend_port}"
    echo -e ""
    echo -e "快速命令:"
    echo -e "  ${CYAN}$0 status${NC}        - 查看服务状态"
    echo -e "  ${CYAN}$0 logs${NC}          - 查看所有日志"
    echo -e "  ${CYAN}$0 stop${NC}          - 停止所有服务"
    echo -e ""

    # 尝试打开浏览器
    if command -v xdg-open > /dev/null 2>&1; then
        log_info "正在打开浏览器..."
        xdg-open "http://127.0.0.1:${DEFAULT_FRONTEND_PORT}" 2>/dev/null || true
    fi
}

# 停止开发环境
stop_dev() {
    log_section "停止 Artemis 开发环境"

    stop_frontend
    stop_backend

    log_info "开发环境已停止"
}

# 重启开发环境
restart_dev() {
    log_section "重启 Artemis 开发环境"

    stop_dev
    sleep 2
    start_dev "$@"
}

# 查看状态
status_dev() {
    log_section "Artemis 开发环境状态"

    echo -e "${CYAN}后端服务状态:${NC}"
    "${CLUSTER_SCRIPT}" status

    echo -e "\n${CYAN}前端服务状态:${NC}"
    if [ -f "${FRONTEND_PID_FILE}" ]; then
        local pid=$(cat "${FRONTEND_PID_FILE}")
        if kill -0 "${pid}" 2>/dev/null; then
            if curl -s -f "http://127.0.0.1:${DEFAULT_FRONTEND_PORT}" > /dev/null 2>&1; then
                echo -e "前端服务: ${GREEN}运行中${NC} (PID: ${pid}, 端口: ${DEFAULT_FRONTEND_PORT})"
            else
                echo -e "前端服务: ${YELLOW}启动中${NC} (PID: ${pid}, 端口: ${DEFAULT_FRONTEND_PORT})"
            fi
        else
            echo -e "前端服务: ${RED}已停止${NC} (PID 文件存在但进程不存在)"
        fi
    else
        echo -e "前端服务: ${RED}未运行${NC}"
    fi
}

# 查看日志
logs_dev() {
    local target=${1:-"all"}

    case "${target}" in
        backend)
            log_info "查看后端日志..."
            "${CLUSTER_SCRIPT}" logs
            ;;
        frontend)
            log_info "查看前端日志..."
            if [ -f "${DEV_DIR}/frontend.log" ]; then
                tail -f "${DEV_DIR}/frontend.log"
            else
                log_error "前端日志文件不存在"
                return 1
            fi
            ;;
        all|*)
            log_info "查看所有日志 (Ctrl+C 退出)..."
            # 使用 tail -f 同时查看多个日志
            if [ -f "${DEV_DIR}/frontend.log" ]; then
                tail -f "${SCRIPT_DIR}/.cluster/logs"/*.log "${DEV_DIR}/frontend.log" 2>/dev/null || \
                tail -f "${DEV_DIR}/frontend.log"
            else
                "${CLUSTER_SCRIPT}" logs
            fi
            ;;
    esac
}

# 清理
clean_dev() {
    log_section "清理开发环境"

    # 停止服务
    stop_dev

    # 清理后端
    "${CLUSTER_SCRIPT}" clean

    # 清理开发目录
    if [ -d "${DEV_DIR}" ]; then
        rm -rf "${DEV_DIR}"
        log_info "开发目录已清理"
    fi

    log_info "清理完成"
}

# 显示帮助
show_help() {
    cat <<EOF
Artemis 开发环境一键启动脚本

用法: $0 <命令> [选项]

命令:
    start [节点数] [后端端口]
        启动开发环境 (后端集群 + 前端 + SQLite 数据库)
        默认: 3 节点后端集群, 端口 8080, SQLite 数据库
        前端端口固定为 5173
        示例:
          $0 start              # 3节点集群 + 前端 + SQLite (默认)
          $0 start 1            # 1节点后端 + 前端
          $0 start 5 9000       # 5节点集群(9000起) + 前端

    stop
        停止所有服务 (后端 + 前端)

    restart [节点数] [后端端口]
        重启开发环境

    status
        查看服务状态

    logs [target]
        查看日志
        target: all (默认), backend, frontend
        示例:
          $0 logs              # 所有日志
          $0 logs backend      # 只看后端
          $0 logs frontend     # 只看前端

    clean
        停止服务并清理所有文件

    help
        显示此帮助信息

服务访问:
    后端 API:  http://127.0.0.1:8080
    前端界面:  http://127.0.0.1:5173

日志位置:
    后端日志:  ${SCRIPT_DIR}/.cluster/logs/
    前端日志:  ${DEV_DIR}/frontend.log

环境变量:
    DB_TYPE         数据库类型: sqlite (默认), mysql, none
    DB_URL          数据库连接 URL (可选)
    DB_MAX_CONN     最大连接数 (默认: 10)

示例:
    # 基本使用 (默认: 3节点 + SQLite)
    $0 start                    # 启动开发环境
    $0 status                   # 查看状态
    $0 logs                     # 查看日志
    $0 stop                     # 停止服务

    # 单节点模式
    $0 start 1                  # 启动1节点

    # 大型集群
    $0 start 5                  # 启动5节点集群

    # 使用 MySQL 数据库
    DB_TYPE=mysql DB_URL="mysql://user:pass@localhost:3306/artemis" $0 start

    # 纯内存模式（无数据库）
    DB_TYPE=none $0 start
EOF
}

# 主函数
main() {
    local command=${1:-"help"}
    shift || true

    case "${command}" in
        start)
            start_dev "$@"
            ;;
        stop)
            stop_dev
            ;;
        restart)
            restart_dev "$@"
            ;;
        status)
            status_dev
            ;;
        logs)
            logs_dev "$@"
            ;;
        clean)
            clean_dev
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
