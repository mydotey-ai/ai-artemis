# Artemis in Rust

10 years ago, I wrote the Artemis service registry in Java at ctrip.com. Artemis is like Eureka which is famous as Service Registry in microservices.

github repo: [artemis](https://github.com/mydotey/artemis)

Artemis in Java has performance issues like long GC stop when hosting large numbers of service instances. In the current repo 'ai-artemis', I want to rewrite Artemis in Rust.

There is a rewrite research doc: docs/artemis-rust-rewrite-specification.md. If any issue is not clarified, please research the original github repo.
