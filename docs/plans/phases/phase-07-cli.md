# 阶段7: artemis CLI实现

> **For Claude:** CLI工具和服务器启动程序。参考Java实现: `artemis-java/artemis-package/`

**优先级**: P0 (必须完成)
**状态**: ✅ **已完成** (2026-02-13)
**目标:** 实现完整的CLI工具
**任务数:** 4个Task

---

## Task 7.1: 实现CLI主入口和server子命令

**Files:**
- Create: `artemis/src/main.rs`
- Create: `artemis/src/cli.rs`
- Create: `artemis/src/commands/mod.rs`
- Create: `artemis/src/commands/server.rs`

**Step 1: 实现CLI结构**

```rust
// artemis/src/cli.rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "artemis")]
#[command(version, about = "Artemis Service Registry", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 启动Artemis服务器
    Server {
        /// 配置文件路径
        #[arg(short, long, default_value = "artemis.toml")]
        config: String,
    },
    /// 服务管理命令
    Service {
        #[command(subcommand)]
        action: ServiceAction,
    },
    /// 实例管理命令
    Instance {
        #[command(subcommand)]
        action: InstanceAction,
    },
}

#[derive(Subcommand)]
pub enum ServiceAction {
    /// 列出所有服务
    List {
        #[arg(short, long)]
        region: String,
        #[arg(short, long)]
        zone: String,
    },
    /// 查询服务详情
    Get {
        #[arg(short, long)]
        service_id: String,
    },
}

#[derive(Subcommand)]
pub enum InstanceAction {
    /// 注册实例
    Register {
        #[arg(short, long)]
        service_id: String,
        #[arg(short, long)]
        ip: String,
        #[arg(short, long)]
        port: u16,
    },
    /// 注销实例
    Unregister {
        #[arg(short, long)]
        instance_id: String,
    },
}
```

**Step 2: 实现server命令**

```rust
// artemis/src/commands/mod.rs
pub mod server;

pub use server::run_server;
```

```rust
// artemis/src/commands/server.rs
use anyhow::Result;
use artemis_core::config::ArtemisConfig;
use artemis_server::{
    cache::VersionedCacheManager, discovery::DiscoveryServiceImpl, lease::LeaseManager,
    ratelimiter::RateLimiter, registry::RegistryRepository, registry::RegistryServiceImpl,
};
use artemis_web::{AppState, WebServer};
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use std::sync::Arc;
use tracing::info;

pub async fn run_server(config_path: &str) -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt::init();

    // 加载配置
    let config: ArtemisConfig = Figment::new()
        .merge(Toml::file(config_path))
        .merge(Env::prefixed("ARTEMIS_"))
        .extract()?;

    info!("Starting Artemis Server with config: {:?}", config);

    // 初始化组件
    let repository = RegistryRepository::new();
    let lease_manager = Arc::new(LeaseManager::new(config.registry.lease_ttl));
    let cache = Arc::new(VersionedCacheManager::new());
    let rate_limiter = RateLimiter::new(config.registry.rate_limit_rps);

    // 创建服务
    let registry_service = RegistryServiceImpl::new(repository.clone(), lease_manager.clone());
    let discovery_service = DiscoveryServiceImpl::new(repository.clone(), cache.clone());

    // 启动租约清理任务
    let repo_clone = repository.clone();
    lease_manager.clone().start_eviction_task(
        config.registry.eviction_interval,
        move |key| {
            info!("Evicting instance: {:?}", key);
            repo_clone.remove(&key);
        },
    );

    // 创建Web服务器
    let app_state = AppState::new(registry_service, discovery_service, rate_limiter, cache);
    let server = WebServer::new(&config.server.host, config.server.port, app_state);

    // 启动服务器
    server.run().await?;

    Ok(())
}
```

**Step 3: 实现main.rs**

```rust
// artemis/src/main.rs
mod cli;
mod commands;

use clap::Parser;
use cli::{Cli, Commands};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Server { config } => {
            commands::run_server(&config).await?;
        }
        Commands::Service { action } => {
            println!("Service command not yet implemented: {:?}", action);
        }
        Commands::Instance { action } => {
            println!("Instance command not yet implemented: {:?}", action);
        }
    }

    Ok(())
}
```

**Step 4: 创建示例配置文件**

```toml
# artemis.toml.example
[server]
host = "0.0.0.0"
port = 8080
region_id = "us-east"
zone_id = "zone-1"

[registry]
lease_ttl = "30s"
eviction_interval = "10s"
rate_limit_rps = 1000

[cluster]
enabled = false
```

**Step 5: 验证编译**

```bash
cargo build -p artemis --release
```

Expected: 编译成功

**Step 6: 提交**

```bash
git add artemis/
git commit -m "feat(cli): implement server command

- Add CLI structure with clap
- Implement server subcommand
- Load config from TOML file
- Initialize all components and start web server
- Add example configuration file

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 7.2: 实现service和instance子命令

**Files:**
- Create: `artemis/src/commands/service.rs`
- Create: `artemis/src/commands/instance.rs`

**Step 1: 实现service命令**

```rust
// artemis/src/commands/service.rs
use anyhow::Result;
use artemis_core::model::{GetServicesRequest, GetServiceRequest, DiscoveryConfig};
use reqwest::Client;

pub async fn list_services(server_url: &str, region: &str, zone: &str) -> Result<()> {
    let client = Client::new();
    let url = format!("{}/api/discovery/getservices", server_url);

    let request = GetServicesRequest {
        region_id: region.to_string(),
        zone_id: zone.to_string(),
    };

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await?
        .json::<artemis_core::model::GetServicesResponse>()
        .await?;

    println!("Services:");
    for service in response.services {
        println!("  - {} ({} instances)", service.service_id, service.instances.len());
    }

    Ok(())
}

pub async fn get_service(server_url: &str, service_id: &str, region: &str, zone: &str) -> Result<()> {
    let client = Client::new();
    let url = format!("{}/api/discovery/getservice", server_url);

    let request = GetServiceRequest {
        discovery_config: DiscoveryConfig {
            service_id: service_id.to_string(),
            region_id: region.to_string(),
            zone_id: zone.to_string(),
            discovery_data: None,
        },
    };

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await?
        .json::<artemis_core::model::GetServiceResponse>()
        .await?;

    if let Some(service) = response.service {
        println!("Service: {}", service.service_id);
        println!("Instances:");
        for instance in service.instances {
            println!("  - {} ({}:{})", instance.instance_id, instance.ip, instance.port);
        }
    } else {
        println!("Service not found");
    }

    Ok(())
}
```

**Step 2: 实现instance命令**

```rust
// artemis/src/commands/instance.rs
use anyhow::Result;
use artemis_core::model::{Instance, InstanceStatus, RegisterRequest, UnregisterRequest, InstanceKey};
use reqwest::Client;

pub async fn register_instance(
    server_url: &str,
    service_id: &str,
    ip: &str,
    port: u16,
    region: &str,
    zone: &str,
) -> Result<()> {
    let client = Client::new();
    let url = format!("{}/api/registry/register", server_url);

    let instance = Instance {
        region_id: region.to_string(),
        zone_id: zone.to_string(),
        group_id: None,
        service_id: service_id.to_string(),
        instance_id: format!("{}:{}", ip, port),
        machine_name: None,
        ip: ip.to_string(),
        port,
        protocol: Some("http".to_string()),
        url: format!("http://{}:{}", ip, port),
        health_check_url: None,
        status: InstanceStatus::Up,
        metadata: None,
    };

    let request = RegisterRequest {
        instances: vec![instance],
    };

    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await?
        .json::<artemis_core::model::RegisterResponse>()
        .await?;

    if response.response_status.error_code == artemis_core::model::ErrorCode::Success {
        println!("Instance registered successfully");
    } else {
        println!("Registration failed: {:?}", response.response_status.error_message);
    }

    Ok(())
}

pub async fn unregister_instance(
    server_url: &str,
    instance_id: &str,
    service_id: &str,
    region: &str,
    zone: &str,
) -> Result<()> {
    let client = Client::new();
    let url = format!("{}/api/registry/unregister", server_url);

    let key = InstanceKey {
        region_id: region.to_string(),
        zone_id: zone.to_string(),
        service_id: service_id.to_string(),
        group_id: String::new(),
        instance_id: instance_id.to_string(),
    };

    let request = UnregisterRequest {
        instance_keys: vec![key],
    };

    client.post(&url).json(&request).send().await?;
    println!("Instance unregistered successfully");

    Ok(())
}
```

**Step 3: 更新commands/mod.rs**

```rust
// artemis/src/commands/mod.rs
pub mod instance;
pub mod server;
pub mod service;

pub use server::run_server;
```

**Step 4: 更新main.rs以使用新命令**

```rust
// artemis/src/main.rs (更新部分)
use cli::{Cli, Commands, ServiceAction, InstanceAction};

const DEFAULT_SERVER_URL: &str = "http://localhost:8080";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Server { config } => {
            commands::run_server(&config).await?;
        }
        Commands::Service { action } => match action {
            ServiceAction::List { region, zone } => {
                commands::service::list_services(DEFAULT_SERVER_URL, &region, &zone).await?;
            }
            ServiceAction::Get { service_id } => {
                commands::service::get_service(
                    DEFAULT_SERVER_URL,
                    &service_id,
                    "default",
                    "default",
                )
                .await?;
            }
        },
        Commands::Instance { action } => match action {
            InstanceAction::Register { service_id, ip, port } => {
                commands::instance::register_instance(
                    DEFAULT_SERVER_URL,
                    &service_id,
                    &ip,
                    port,
                    "default",
                    "default",
                )
                .await?;
            }
            InstanceAction::Unregister { instance_id } => {
                commands::instance::unregister_instance(
                    DEFAULT_SERVER_URL,
                    &instance_id,
                    "unknown",
                    "default",
                    "default",
                )
                .await?;
            }
        },
    }

    Ok(())
}
```

**Step 5: 提交**

```bash
git add artemis/src/
git commit -m "feat(cli): implement service and instance commands

- Add service list/get subcommands
- Add instance register/unregister subcommands
- HTTP client for API calls
- User-friendly output

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 7.3: 实现配置转换工具

**Files:**
- Create: `artemis/src/commands/config.rs`
- Update: `artemis/src/commands/mod.rs`
- Update: `artemis/src/cli.rs`
- Update: `artemis/src/main.rs`

**Step 1: 添加config子命令到CLI**

```rust
// artemis/src/cli.rs (添加)
#[derive(Subcommand)]
pub enum Commands {
    // ... 现有命令 ...

    /// 配置管理命令
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// 从Java配置文件转换为Rust配置
    Convert {
        /// Java配置文件路径 (application.properties)
        #[arg(short, long)]
        input: String,

        /// 输出Rust配置文件路径 (artemis.toml)
        #[arg(short, long, default_value = "artemis.toml")]
        output: String,
    },

    /// 验证配置文件
    Validate {
        /// 配置文件路径
        #[arg(short, long)]
        config: String,
    },
}
```

**Step 2: 实现config命令处理**

```rust
// artemis/src/commands/config.rs
use anyhow::Result;
use artemis_core::config::ArtemisConfig;
use figment::{providers::Format, Figment};
use std::collections::HashMap;
use std::fs;

/// 从Java properties文件转换为TOML配置
pub async fn convert_java_config(input: &str, output: &str) -> Result<()> {
    println!("Converting Java config from {} to {}", input, output);

    // 读取Java properties文件
    let properties = read_java_properties(input)?;

    // 转换为Rust配置
    let rust_config = convert_properties_to_rust_config(&properties)?;

    // 序列化为TOML
    let toml_content = toml::to_string_pretty(&rust_config)?;

    // 写入输出文件
    fs::write(output, toml_content)?;

    println!("Configuration converted successfully!");
    println!("\nNext steps:");
    println!("1. Review the generated {} file", output);
    println!("2. Adjust any settings as needed");
    println!("3. Start Artemis with: artemis server --config {}", output);

    Ok(())
}

/// 读取Java properties文件
fn read_java_properties(path: &str) -> Result<HashMap<String, String>> {
    let content = fs::read_to_string(path)?;
    let mut properties = HashMap::new();

    for line in content.lines() {
        let line = line.trim();

        // 跳过注释和空行
        if line.is_empty() || line.starts_with('#') || line.starts_with('!') {
            continue;
        }

        // 解析key=value
        if let Some((key, value)) = line.split_once('=') {
            properties.insert(key.trim().to_string(), value.trim().to_string());
        }
    }

    Ok(properties)
}

/// 将Java properties转换为Rust配置
fn convert_properties_to_rust_config(
    props: &HashMap<String, String>,
) -> Result<ArtemisConfig> {
    let mut config = ArtemisConfig::default();

    // Server配置映射
    if let Some(host) = props.get("server.host") {
        config.server.host = host.clone();
    }
    if let Some(port) = props.get("server.port") {
        config.server.port = port.parse()?;
    }
    if let Some(region) = props.get("artemis.region") {
        config.server.region_id = region.clone();
    }
    if let Some(zone) = props.get("artemis.zone") {
        config.server.zone_id = zone.clone();
    }

    // Registry配置映射
    if let Some(ttl) = props.get("artemis.lease.ttl") {
        config.registry.lease_ttl = parse_duration(ttl)?;
    }
    if let Some(interval) = props.get("artemis.eviction.interval") {
        config.registry.eviction_interval = parse_duration(interval)?;
    }
    if let Some(rps) = props.get("artemis.ratelimit.rps") {
        config.registry.rate_limit_rps = rps.parse()?;
    }

    // Cluster配置映射
    if let Some(enabled) = props.get("artemis.cluster.enabled") {
        config.cluster.enabled = enabled.parse()?;
    }
    if let Some(peers) = props.get("artemis.cluster.peers") {
        let peer_list: Vec<String> = peers.split(',').map(|s| s.trim().to_string()).collect();
        config.cluster.peer_nodes = Some(peer_list);
    }

    // Database配置映射
    if let Some(db_url) = props.get("artemis.db.url") {
        config.database = Some(artemis_core::config::DatabaseConfig {
            url: db_url.clone(),
            max_connections: props
                .get("artemis.db.max_connections")
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
        });
    }

    Ok(config)
}

/// 解析时间字符串 (如 "30s", "1m", "2h")
fn parse_duration(s: &str) -> Result<std::time::Duration> {
    let s = s.trim();

    if s.ends_with("ms") {
        let num: u64 = s[..s.len() - 2].parse()?;
        Ok(std::time::Duration::from_millis(num))
    } else if s.ends_with('s') {
        let num: u64 = s[..s.len() - 1].parse()?;
        Ok(std::time::Duration::from_secs(num))
    } else if s.ends_with('m') {
        let num: u64 = s[..s.len() - 1].parse()?;
        Ok(std::time::Duration::from_secs(num * 60))
    } else if s.ends_with('h') {
        let num: u64 = s[..s.len() - 1].parse()?;
        Ok(std::time::Duration::from_secs(num * 3600))
    } else {
        // 默认假设是秒
        Ok(std::time::Duration::from_secs(s.parse()?))
    }
}

/// 验证配置文件
pub async fn validate_config(path: &str) -> Result<()> {
    println!("Validating configuration file: {}", path);

    // 尝试加载配置
    let _config: ArtemisConfig = Figment::new()
        .merge(figment::providers::Toml::file(path))
        .extract()?;

    println!("✓ Configuration is valid");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("30s").unwrap(), std::time::Duration::from_secs(30));
        assert_eq!(parse_duration("5m").unwrap(), std::time::Duration::from_secs(300));
        assert_eq!(parse_duration("2h").unwrap(), std::time::Duration::from_secs(7200));
        assert_eq!(parse_duration("500ms").unwrap(), std::time::Duration::from_millis(500));
    }
}
```

**Step 3: 更新main.rs处理config命令**

```rust
// artemis/src/main.rs
Commands::Config { action } => match action {
    ConfigAction::Convert { input, output } => {
        commands::config::convert_java_config(&input, &output).await?;
    }
    ConfigAction::Validate { config } => {
        commands::config::validate_config(&config).await?;
    }
},
```

**Step 4: 更新commands/mod.rs**

```rust
// artemis/src/commands/mod.rs
pub mod config;
pub mod instance;
pub mod server;
pub mod service;

pub use server::run_server;
```

**Step 5: 创建Java配置示例文件**

```properties
# application.properties.example
# Server配置
server.host=0.0.0.0
server.port=8080

# Artemis配置
artemis.region=us-east
artemis.zone=zone-1

# Registry配置
artemis.lease.ttl=30s
artemis.eviction.interval=10s
artemis.ratelimit.rps=1000

# Cluster配置
artemis.cluster.enabled=false
# artemis.cluster.peers=node1:8080,node2:8080

# Database配置 (可选)
# artemis.db.url=mysql://user:pass@localhost/artemis
# artemis.db.max_connections=10
```

**Step 6: 测试配置转换**

```bash
# 创建测试用的Java配置
cat > test.properties <<EOF
server.port=9090
artemis.region=cn-east
artemis.lease.ttl=60s
EOF

# 运行转换
cargo run -p artemis -- config convert --input test.properties --output test.toml

# 验证生成的配置
cargo run -p artemis -- config validate --config test.toml

# 清理
rm test.properties test.toml
```

Expected: 转换成功，验证通过

**Step 7: 提交**

```bash
git add artemis/
git commit -m "feat(cli): implement config conversion tool

- Add config subcommand with convert and validate actions
- Support Java properties to Rust TOML conversion
- Parse duration strings (30s, 5m, 2h)
- Map Java config keys to Rust config structure
- Add validation for converted configs
- Add example Java properties file

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 7.4: 添加使用文档

**Files:**
- Create: `artemis/README.md`

**Step 1: 创建README**

```markdown
# Artemis CLI

Artemis服务注册中心的命令行工具。

## 安装

\`\`\`bash
cargo build --release -p artemis
\`\`\`

可执行文件位于 `target/release/artemis`

## 使用

### 启动服务器

\`\`\`bash
artemis server --config artemis.toml
\`\`\`

### 服务管理

列出所有服务:
\`\`\`bash
artemis service list --region us-east --zone zone-1
\`\`\`

查询服务详情:
\`\`\`bash
artemis service get --service-id my-service
\`\`\`

### 实例管理

注册实例:
\`\`\`bash
artemis instance register --service-id my-service --ip 192.168.1.100 --port 8080
\`\`\`

注销实例:
\`\`\`bash
artemis instance unregister --instance-id 192.168.1.100:8080
\`\`\`

## 配置文件

配置文件示例 `artemis.toml`:

\`\`\`toml
[server]
host = "0.0.0.0"
port = 8080
region_id = "us-east"
zone_id = "zone-1"

[registry]
lease_ttl = "30s"
eviction_interval = "10s"
rate_limit_rps = 1000

[cluster]
enabled = false
\`\`\`

## 环境变量

所有配置项都可以通过环境变量覆盖，前缀为 `ARTEMIS_`:

\`\`\`bash
ARTEMIS_SERVER_PORT=9090 artemis server
\`\`\`
```

**Step 2: 提交**

```bash
git add artemis/README.md
git commit -m "docs(cli): add usage documentation

- Document all CLI commands
- Add configuration examples
- Add environment variable usage

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## 阶段7完成标准

- ✅ CLI主入口实现
- ✅ server子命令实现
- ✅ service子命令实现
- ✅ instance子命令实现
- ✅ config子命令实现（转换和验证）
- ✅ 配置文件支持
- ✅ 使用文档
- ✅ `cargo build -p artemis` 成功
