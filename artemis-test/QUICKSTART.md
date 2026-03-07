# Artemis Hybrid Test - Quick Start

## 3-Step Quick Start

### Step 1: Prepare Environment
```bash
cd artemis-test
./scripts/setup.sh
```

### Step 2: Start Cluster
```bash
./scripts/start-cluster.sh
```

### Step 3: Check Status
```bash
./scripts/status.sh
```

## Next Steps

1. **Start Web Console** (optional):
   ```bash
   ./scripts/start-console.sh
   # Open http://localhost:5173
   # Login: admin / admin123
   ```

2. **Implement Test Apps**:
   - See `apps/` directory
   - Implement Java/Rust Providers and Consumers
   - Then run: `./scripts/start-apps.sh`

3. **Run Tests**:
   ```bash
   ./scripts/run-test.sh 600  # 10 minutes
   ```

4. **Cleanup**:
   ```bash
   ./scripts/cleanup.sh
   ```

## Common Commands

```bash
# View all logs
tail -f logs/*.log

# View specific log
tail -f logs/java-node1.log
tail -f logs/rust-node1.log

# Check specific node
curl http://localhost:8081/health
curl http://localhost:8081/api/cluster/status

# Test service discovery
curl "http://localhost:8081/api/discovery/instances?serviceId=hybrid-test-hello-service"
```

## Port Reference

| Service | Port |
|---------|------|
| Java Node 1 | 8081 |
| Java Node 2 | 8082 |
| Java Node 3 | 8083 |
| Rust Node 1 | 8084 |
| Rust Node 2 | 8085 |
| Rust Node 3 | 8086 |
| Java Provider 1 | 8087 |
| Java Provider 2 | 8088 |
| Rust Provider 1 | 8089 |
| Rust Provider 2 | 8090 |
| Web Console | 5173 |

## Troubleshooting

**Port already in use:**
```bash
# Find process
lsof -i :8081

# Kill process
kill -9 <PID>

# Or use cleanup
./scripts/cleanup.sh
```

**Java node fails to start:**
```bash
# Check Java version
java -version

# Check logs
cat logs/java-node1.log
```

**Rust node fails to start:**
```bash
# Rebuild
cd .. && cargo build --release

# Check logs
cat logs/rust-node1.log
```
