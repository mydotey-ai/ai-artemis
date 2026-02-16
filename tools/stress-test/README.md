# Artemis 压力测试工具

高性能压力测试工具,用于验证 Artemis 服务器在高负载下的性能表现。

## 功能特性

- ✅ **高并发**: 支持 100+ 并发客户端
- ✅ **高 QPS**: 支持 10,000+ QPS 压力测试
- ✅ **延迟统计**: P50/P90/P95/P99/P99.9 延迟分布
- ✅ **吞吐量监控**: 实时 QPS 统计
- ✅ **错误率统计**: 成功/失败请求统计
- ✅ **实时进度**: 进度条显示测试进度
- ✅ **多种模式**: register, heartbeat, discovery, mixed

## 安装

```bash
cd tools/stress-test
cargo build --release
```

## 使用方法

### 基础用法

```bash
# 默认配置: 100并发, 10,000 QPS, 60秒, mixed模式
./target/release/artemis-stress-test

# 指定服务器地址
./target/release/artemis-stress-test --url http://localhost:8080

# 自定义并发和QPS
./target/release/artemis-stress-test --concurrency 200 --qps 20000

# 自定义测试时长
./target/release/artemis-stress-test --duration 300
```

### 测试模式

```bash
# 纯注册压力测试
./target/release/artemis-stress-test --mode register --qps 5000

# 纯心跳压力测试
./target/release/artemis-stress-test --mode heartbeat --qps 15000

# 混合模式 (90% 心跳 + 10% 注册)
./target/release/artemis-stress-test --mode mixed --qps 10000
```

### 完整参数

```bash
./target/release/artemis-stress-test \
  --url http://localhost:8080 \
  --concurrency 100 \
  --qps 10000 \
  --duration 60 \
  --instances-per-client 10 \
  --mode mixed
```

## 参数说明

| 参数 | 简写 | 默认值 | 说明 |
|------|------|--------|------|
| `--url` | `-u` | `http://localhost:8080` | Artemis 服务器地址 |
| `--concurrency` | `-c` | `100` | 并发客户端数量 |
| `--qps` | `-q` | `10000` | 目标 QPS (每秒请求数) |
| `--duration` | `-d` | `60` | 测试持续时间 (秒) |
| `--instances-per-client` | `-i` | `10` | 每个客户端的实例数 |
| `--mode` | `-m` | `mixed` | 测试模式: register, heartbeat, mixed |

## 输出示例

```
=== Artemis 压力测试启动 ===
服务器: http://localhost:8080
并发数: 100
目标 QPS: 10000
持续时间: 60 秒
每客户端实例数: 10
测试模式: mixed

⠁ [00:01:00] [########################################] 60/60s (完成!)

=== 压力测试结果 ===
总请求数: 600000
成功请求: 599998 (99.99%)
失败请求: 2 (0.01%)
实际 QPS: 10000.00

=== 延迟分布 (微秒) ===
P50:   5.2 µs
P90:   7.8 µs
P95:   9.1 µs
P99:   12.5 µs
P99.9: 18.3 µs
Min:   3.1 µs
Max:   125.7 µs
Mean:  6.23 µs
```

## 性能目标

### 短期目标 (Week 1)

- ✅ 10,000 QPS 稳定运行 60 秒
- ✅ P99 延迟 < 10 µs
- ✅ 错误率 < 0.1%
- ✅ 100 并发客户端

### 中期目标 (Month 1)

- ⏳ 50,000 QPS 稳定运行 5 分钟
- ⏳ P99 延迟 < 20 µs
- ⏳ 错误率 < 0.01%
- ⏳ 500 并发客户端

### 长期目标 (Month 3)

- ⏳ 100,000 QPS 稳定运行 10 分钟
- ⏳ P99 延迟 < 50 µs
- ⏳ 错误率 < 0.001%
- ⏳ 1000+ 并发客户端

## 监控指标

测试过程中建议监控:

1. **服务器 CPU 使用率**
   ```bash
   top -p $(pgrep artemis)
   ```

2. **服务器内存使用**
   ```bash
   ps aux | grep artemis
   ```

3. **Prometheus 指标**
   ```bash
   curl http://localhost:8080/metrics | grep artemis
   ```

4. **网络流量**
   ```bash
   iftop -i eth0
   ```

## 故障排查

### 连接失败

```bash
# 检查服务器是否运行
curl http://localhost:8080/health

# 检查端口是否开放
netstat -tuln | grep 8080
```

### 延迟过高

- 检查服务器 CPU 是否饱和
- 检查网络延迟 (`ping localhost`)
- 减少并发数或 QPS

### 错误率过高

- 查看服务器日志
- 检查是否触发限流
- 检查数据库连接池

## 开发

```bash
# 编译
cargo build --release

# 运行
cargo run --release -- --help

# 测试
cargo test
```

## 许可证

MIT OR Apache-2.0
