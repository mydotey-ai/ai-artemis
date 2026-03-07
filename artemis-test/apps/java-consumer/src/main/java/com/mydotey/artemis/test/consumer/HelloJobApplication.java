package com.mydotey.artemis.test.consumer;

import com.mydotey.artemis.ArtemisClientManager;
import com.mydotey.artemis.DiscoveryClient;
import com.mydotey.artemis.model.Instance;
import org.apache.http.client.methods.CloseableHttpResponse;
import org.apache.http.client.methods.HttpGet;
import org.apache.http.impl.client.CloseableHttpClient;
import org.apache.http.impl.client.HttpClients;
import org.apache.http.util.EntityUtils;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.boot.CommandLineRunner;
import org.springframework.boot.SpringApplication;
import org.springframework.boot.autoconfigure.SpringBootApplication;
import org.springframework.scheduling.annotation.EnableScheduling;
import org.springframework.scheduling.annotation.Scheduled;

import javax.annotation.PreDestroy;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;

@SpringBootApplication
@EnableScheduling
public class HelloJobApplication implements CommandLineRunner {

    @Value("${artemis.servers:http://localhost:8081}")
    private String artemisServers;

    @Value("${consumer.id:java-consumer-1}")
    private String consumerId;

    private DiscoveryClient discoveryClient;
    private CloseableHttpClient httpClient;
    private final AtomicInteger roundRobin = new AtomicInteger(0);

    public static void main(String[] args) {
        SpringApplication.run(HelloJobApplication.class, args);
    }

    @Override
    public void run(String... args) throws Exception {
        // 初始化 HTTP 客户端
        httpClient = HttpClients.createDefault();

        // 初始化 Artemis 客户端
        String[] servers = artemisServers.split(",");
        ArtemisClientManager manager = new ArtemisClientManager();
        manager.setServerUrlList(servers);

        discoveryClient = manager.getDiscoveryClient();

        System.out.println("[" + consumerId + "] Started. Artemis servers: " + artemisServers);
    }

    @Scheduled(fixedRate = 200)
    public void callHelloService() {
        try {
            // 发现服务实例
            List<Instance> instances = discoveryClient.getService("hybrid-test-hello-service");

            if (instances == null || instances.isEmpty()) {
                System.out.println("[" + consumerId + "] No instances available");
                return;
            }

            // 简单的轮询负载均衡
            int index = roundRobin.getAndIncrement() % instances.size();
            Instance target = instances.get(index);

            // 调用服务
            String url = String.format("http://%s:%d/sayHello",
                target.getHost(), target.getPort());

            long startTime = System.currentTimeMillis();

            HttpGet request = new HttpGet(url);
            try (CloseableHttpResponse response = httpClient.execute(request)) {
                int statusCode = response.getStatusLine().getStatusCode();
                long latency = System.currentTimeMillis() - startTime;

                if (statusCode == 200) {
                    String body = EntityUtils.toString(response.getEntity());
                    System.out.printf("[%s] Target=%s:%d Response=%s Latency=%dms%n",
                        consumerId, target.getHost(), target.getPort(), body, latency);
                } else {
                    System.err.printf("[%s] HTTP %d from %s:%d%n",
                        consumerId, statusCode, target.getHost(), target.getPort());
                }
            }

        } catch (Exception e) {
            System.err.println("[" + consumerId + "] Error: " + e.getMessage());
        }
    }

    @PreDestroy
    public void shutdown() {
        try {
            if (httpClient != null) {
                httpClient.close();
            }
            System.out.println("[" + consumerId + "] Shutdown complete");
        } catch (Exception e) {
            e.printStackTrace();
        }
    }
}
