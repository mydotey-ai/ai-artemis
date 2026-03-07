package com.mydotey.artemis.test.provider;

import com.mydotey.artemis.ArtemisClientManager;
import com.mydotey.artemis.RegistryClient;
import com.mydotey.artemis.model.Instance;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RestController;

import javax.annotation.PostConstruct;
import java.time.LocalDateTime;
import java.time.format.DateTimeFormatter;
import java.util.HashMap;
import java.util.Map;

@SpringBootApplication
@RestController
public class HelloServiceApplication {

    @Value("${server.port}")
    private int port;

    @Value("${artemis.servers:http://localhost:8081}")
    private String artemisServers;

    private RegistryClient registryClient;

    public static void main(String[] args) {
        SpringApplication.run(HelloServiceApplication.class, args);
    }

    @GetMapping("/sayHello")
    public String sayHello() {
        String timestamp = LocalDateTime.now().format(DateTimeFormatter.ISO_LOCAL_DATE_TIME);
        return String.format("Hello from Java [%d] at %s", port, timestamp);
    }

    @GetMapping("/health")
    public String health() {
        return "OK";
    }

    @PostConstruct
    public void register() {
        try {
            // 初始化 Artemis 客户端
            String[] servers = artemisServers.split(",");
            ArtemisClientManager manager = new ArtemisClientManager();
            manager.setServerUrlList(servers);

            registryClient = manager.getRegistryClient();

            // 构建实例信息
            Instance instance = new Instance();
            instance.setServiceId("hybrid-test-hello-service");
            instance.setInstanceId("java-web-" + port);
            instance.setHost("127.0.0.1");
            instance.setPort(port);
            instance.setHealthCheckUrl("http://127.0.0.1:" + port + "/health");

            Map<String, String> metadata = new HashMap<>();
            metadata.put("app", "java-provider");
            metadata.put("language", "java");
            instance.setMetadata(metadata);

            // 注册服务
            registryClient.register(instance);

            System.out.println("[Java Provider] Registered to Artemis: port=" + port + ", servers=" + artemisServers);

        } catch (Exception e) {
            System.err.println("[Java Provider] Failed to register: " + e.getMessage());
            e.printStackTrace();
        }
    }
}
