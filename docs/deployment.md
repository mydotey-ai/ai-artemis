# Artemis 部署指南

## 快速开始

### 本地运行

```bash
# 编译
cargo build --release -p artemis

# 启动服务器
./target/release/artemis server --addr 0.0.0.0:8080
```

## Docker部署

### 构建镜像

```bash
docker build -t artemis:latest .
```

构建时间: 约5-10分钟(首次构建)

### 运行容器

基础运行:

```bash
docker run -d \
  --name artemis \
  -p 8080:8080 \
  artemis:latest
```

带环境变量:

```bash
docker run -d \
  --name artemis \
  -p 8080:8080 \
  -e RUST_LOG=info \
  artemis:latest
```

### Docker Compose

创建`docker-compose.yml`:

```yaml
version: '3.8'

services:
  artemis:
    image: artemis:latest
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
```

启动:

```bash
docker-compose up -d
```

## 性能调优

### 推荐配置

**最小配置:**
- CPU: 1核心
- 内存: 512MB
- 磁盘: 1GB

**生产配置(10万实例):**
- CPU: 2-4核心
- 内存: 2-4GB
- 磁盘: 5GB
- 网络: 1Gbps

### 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `RUST_LOG` | 日志级别 | `info` |
| `RUST_BACKTRACE` | 启用堆栈跟踪 | `0` |

## 监控

### Prometheus

Artemis暴露Prometheus指标在`/metrics`端点。

**抓取配置:**

```yaml
scrape_configs:
  - job_name: 'artemis'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

**关键指标:**

- `artemis_register_requests_total` - 注册请求总数
- `artemis_heartbeat_requests_total` - 心跳请求总数
- `artemis_discovery_requests_total` - 发现请求总数
- `artemis_active_instances` - 活跃实例数

### 健康检查

```bash
curl http://localhost:8080/health
```

预期响应: `OK` (HTTP 200)

### 日志

查看容器日志:

```bash
docker logs -f artemis
```

日志级别:
- `error` - 仅错误
- `warn` - 警告和错误
- `info` - 信息、警告和错误(推荐)
- `debug` - 调试信息(开发)
- `trace` - 详细跟踪(性能调试)

## 高可用部署

### 多实例部署

Artemis支持无状态水平扩展。使用负载均衡器分发流量:

```yaml
version: '3.8'

services:
  artemis-1:
    image: artemis:latest
    ports:
      - "8081:8080"
    environment:
      - RUST_LOG=info

  artemis-2:
    image: artemis:latest
    ports:
      - "8082:8080"
    environment:
      - RUST_LOG=info

  artemis-3:
    image: artemis:latest
    ports:
      - "8083:8080"
    environment:
      - RUST_LOG=info

  nginx:
    image: nginx:latest
    ports:
      - "8080:80"
    volumes:
      - ./nginx.conf:/etc/nginx/nginx.conf
    depends_on:
      - artemis-1
      - artemis-2
      - artemis-3
```

**Nginx配置示例:**

```nginx
upstream artemis {
    server artemis-1:8080;
    server artemis-2:8080;
    server artemis-3:8080;
}

server {
    listen 80;

    location / {
        proxy_pass http://artemis;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### Kubernetes部署

创建`k8s-deployment.yaml`:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: artemis
spec:
  replicas: 3
  selector:
    matchLabels:
      app: artemis
  template:
    metadata:
      labels:
        app: artemis
    spec:
      containers:
      - name: artemis
        image: artemis:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            memory: "512Mi"
            cpu: "500m"
          limits:
            memory: "2Gi"
            cpu: "2000m"
---
apiVersion: v1
kind: Service
metadata:
  name: artemis-service
spec:
  selector:
    app: artemis
  ports:
  - protocol: TCP
    port: 8080
    targetPort: 8080
  type: LoadBalancer
```

部署:

```bash
kubectl apply -f k8s-deployment.yaml
```

## 故障排除

### 常见问题

**1. 端口被占用**

```bash
# 查看占用端口的进程
lsof -i :8080
# 或
netstat -tlnp | grep 8080
```

**2. 内存不足**

增加容器内存限制:

```bash
docker run -d \
  --name artemis \
  -p 8080:8080 \
  --memory="2g" \
  artemis:latest
```

**3. 连接超时**

检查防火墙规则:

```bash
# Ubuntu/Debian
sudo ufw allow 8080/tcp

# CentOS/RHEL
sudo firewall-cmd --add-port=8080/tcp --permanent
sudo firewall-cmd --reload
```

### 性能问题诊断

```bash
# 查看指标
curl http://localhost:8080/metrics

# 查看系统资源
docker stats artemis

# 查看详细日志
RUST_LOG=debug docker run -p 8080:8080 artemis:latest
```

## 安全建议

1. **不要在生产环境使用默认配置**
2. **启用HTTPS**（使用反向代理如Nginx）
3. **限制网络访问**（使用防火墙规则）
4. **定期更新镜像**
5. **监控异常流量**

## 备份和恢复

Artemis是无状态服务，不需要数据备份。服务实例注册信息存储在内存中，重启后客户端会重新注册。

## 性能基准

在标准配置下(2核CPU, 2GB内存):

- **注册延迟**: P99 < 0.5ms
- **心跳延迟**: P99 < 0.5ms
- **发现延迟**: P99 < 1ms
- **支持实例数**: 100,000+
- **QPS**: 10,000+

详细性能报告: 运行`cargo bench --package artemis-server`

## 许可证

MIT OR Apache-2.0
