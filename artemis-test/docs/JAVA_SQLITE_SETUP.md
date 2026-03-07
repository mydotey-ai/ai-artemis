# Java Artemis SQLite 支持改造方案

## 1. 添加 SQLite 依赖

在 `artemis-java/pom.xml` 中添加 SQLite JDBC 驱动：

```xml
<properties>
    <!-- 在现有属性后添加 -->
    <sqlite-jdbc.version>3.44.1.0</sqlite-jdbc.version>
</properties>

<dependencyManagement>
    <dependencies>
        <!-- 在现有依赖后添加 -->
        <dependency>
            <groupId>org.xerial</groupId>
            <artifactId>sqlite-jdbc</artifactId>
            <version>${sqlite-jdbc.version}</version>
        </dependency>
    </dependencies>
</dependencyManagement>
```

在 `artemis-java/artemis-management/pom.xml` 中添加依赖：

```xml
<dependencies>
    <!-- 在现有依赖后添加 -->
    <dependency>
        <groupId>org.xerial</groupId>
        <artifactId>sqlite-jdbc</artifactId>
    </dependency>
</dependencies>
```

## 2. 创建 SQLite 配置文件

创建 `artemis-java/artemis-package/src/main/resources/data-source-sqlite.properties`：

```properties
# SQLite DataSource Configuration
driverClassName=org.sqlite.JDBC
url=jdbc:sqlite:./artemis.db
username=
password=
initialSize=5
maxActive=20
maxIdle=10
minIdle=2
maxWait=30000
```

## 3. 修改 DataConfig 支持多数据库

修改 `artemis-java/artemis-management/src/main/java/org/mydotey/artemis/management/dao/DataConfig.java`：

```java
package org.mydotey.artemis.management.dao;

import java.io.InputStream;
import java.util.Properties;
import java.util.concurrent.atomic.AtomicBoolean;

import javax.sql.DataSource;

import org.apache.commons.dbcp2.BasicDataSourceFactory;
import org.springframework.jdbc.core.JdbcTemplate;
import org.springframework.jdbc.datasource.DataSourceTransactionManager;

public class DataConfig {

    private static final String DB_CONFIG_FILE = "data-source.properties";
    private static final String DB_CONFIG_FILE_ENV = "ARTEMIS_DB_CONFIG";

    private static DataSource _dataSource;
    private static DataSourceTransactionManager _dataSourceTransactionManager;
    private static JdbcTemplate _jdbcTemplate;

    private static AtomicBoolean _inited = new AtomicBoolean();

    public static void init() throws Exception {
        if (!_inited.compareAndSet(false, true))
            return;

        initDataSource();

        _dataSourceTransactionManager = new DataSourceTransactionManager(_dataSource);
        _jdbcTemplate = new JdbcTemplate(_dataSource);
    }

    private static void initDataSource() throws Exception {
        Properties prop = new Properties();

        // 1. 先尝试从环境变量获取配置文件路径
        String configFile = System.getenv(DB_CONFIG_FILE_ENV);
        if (configFile == null) {
            configFile = DB_CONFIG_FILE;
        }

        // 2. 加载配置文件
        try (InputStream is = Thread.currentThread().getContextClassLoader().getResourceAsStream(configFile)) {
            if (is == null) {
                // 如果没有找到配置文件，使用 SQLite 默认配置
                System.out.println("No " + configFile + " found in classpath, using SQLite default config");
                prop.setProperty("driverClassName", "org.sqlite.JDBC");
                prop.setProperty("url", "jdbc:sqlite:./artemis.db");
                prop.setProperty("initialSize", "5");
                prop.setProperty("maxActive", "20");
            } else {
                prop.load(is);
            }
        }

        // 3. 允许通过系统属性覆盖配置
        if (System.getProperty("artemis.db.url") != null) {
            prop.setProperty("url", System.getProperty("artemis.db.url"));
        }
        if (System.getProperty("artemis.db.driver") != null) {
            prop.setProperty("driverClassName", System.getProperty("artemis.db.driver"));
        }

        _dataSource = BasicDataSourceFactory.createDataSource(prop);

        System.out.println("DataSource initialized: " + prop.getProperty("driverClassName") +
                           " -> " + prop.getProperty("url"));
    }

    public static JdbcTemplate jdbcTemplate() {
        return _jdbcTemplate;
    }

    public static DataSourceTransactionManager dataSourceTransactionManager() {
        return _dataSourceTransactionManager;
    }

    public DataSource dataSource() {
        return _dataSource;
    }
}
```

## 4. 添加 SQLite 初始化脚本

创建 `artemis-java/artemis-management/src/main/resources/sqlite-schema.sql`：

```sql
-- SQLite Schema for Artemis Management

-- Groups table
CREATE TABLE IF NOT EXISTS groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id VARCHAR(255) NOT NULL,
    group_name VARCHAR(255) NOT NULL,
    strategy VARCHAR(50) DEFAULT 'round-robin',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(service_id, group_name)
);

-- Route rules table
CREATE TABLE IF NOT EXISTS route_rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id VARCHAR(255) NOT NULL,
    rule_name VARCHAR(255) NOT NULL,
    match_type VARCHAR(50) NOT NULL,
    match_value TEXT NOT NULL,
    target_group VARCHAR(255) NOT NULL,
    priority INTEGER DEFAULT 0,
    enabled BOOLEAN DEFAULT 1,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(service_id, rule_name)
);

-- Zone operations table
CREATE TABLE IF NOT EXISTS zone_operations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    zone_id VARCHAR(255) NOT NULL,
    operation_type VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL,
    detail TEXT,
    operator VARCHAR(255),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP
);

-- Canary configs table
CREATE TABLE IF NOT EXISTS canary_configs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    service_id VARCHAR(255) NOT NULL,
    config_name VARCHAR(255) NOT NULL,
    canary_type VARCHAR(50) NOT NULL,
    canary_value TEXT NOT NULL,
    percentage INTEGER DEFAULT 0,
    enabled BOOLEAN DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(service_id, config_name)
);

-- Audit logs table
CREATE TABLE IF NOT EXISTS audit_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    operation_type VARCHAR(50) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id VARCHAR(255),
    operation_detail TEXT,
    operator VARCHAR(255),
    operation_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    result VARCHAR(20) DEFAULT 'SUCCESS',
    error_message TEXT
);

-- Indexes for better performance
CREATE INDEX IF NOT EXISTS idx_groups_service ON groups(service_id);
CREATE INDEX IF NOT EXISTS idx_route_rules_service ON route_rules(service_id);
CREATE INDEX IF NOT EXISTS idx_zone_operations_zone ON zone_operations(zone_id);
CREATE INDEX IF NOT EXISTS idx_canary_configs_service ON canary_configs(service_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_time ON audit_logs(operation_time);
```

## 5. 修改 artemis-package 添加 SQLite 配置

在 `artemis-java/artemis-package/src/assembly/assembly.xml` 中添加 SQLite 配置文件：

```xml
<fileSets>
    <!-- 其他 fileSet -->
    <fileSet>
        <directory>src/main/resources</directory>
        <outputPath>config</outputPath>
        <includes>
            <include>data-source.properties</include>
            <include>data-source-sqlite.properties</include>
            <include>log4j.properties</include>
        </includes>
    </fileSet>
</fileSets>
```

## 6. 修改启动脚本支持 SQLite

在 `artemis-test/scripts/start-cluster.sh` 中，对于 Java 节点，添加 SQLite 支持：

```bash
# 启动 Java 节点时，使用 SQLite 配置
java -jar "$JAVA_JAR" \
    --server.port=8081 \
    --artemis.peer.port=9091 \
    --spring.datasource.url="jdbc:sqlite:$DATA_DIR/artemis.db" \
    --spring.datasource.driver-class-name="org.sqlite.JDBC" \
    --artemis.cluster.peers=127.0.0.1:8082,... \
    > "$LOGS_DIR/java-node1.log" 2>&1 &
```

或者通过环境变量指定配置文件：

```bash
export ARTEMIS_DB_CONFIG=data-source-sqlite.properties
java -jar "$JAVA_JAR" ...
```

## 总结

Java 版本的 Artemis 当前**只支持 MySQL**，不支持 SQLite。要让 Java 版本支持 SQLite 需要：

1. **添加 sqlite-jdbc 依赖** (已完成文档)
2. **修改 DataConfig.java** 支持动态数据源配置 (已完成文档)
3. **创建 SQLite 数据库 schema** (已完成文档)
4. **添加配置文件和启动脚本支持** (已完成文档)

这些修改需要改动 artemis-java 项目的源代码。如果暂时不想修改 artemis-java，混合集群方案可以调整为：
- **Java 节点使用 MySQL 数据库**
- **Rust 节点使用 SQLite 数据库**
- **Management 数据不共享** (各自独立存储)
- **通过 Replication API 同步实例数据**