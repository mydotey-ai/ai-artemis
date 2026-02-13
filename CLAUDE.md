# Artemis in Rust

10 years ago, I wrote the Artemis service registry in Java at ctrip.com. Artemis is like Eureka which is famous as Service Registry in microservices.

github repo: [artemis](https://github.com/mydotey/artemis)

Artemis in Java has performance issues like long GC stop when hosting large numbers of service instances. In the current repo 'ai-artemis', I want to rewrite Artemis in Rust.

## Documentation

- **Product Specification**: `docs/artemis-rust-rewrite-specification.md` - Complete product requirements and specifications for the Rust rewrite
- **Detailed Design**: `docs/plans/2026-02-13-artemis-rust-design.md` - Comprehensive design document including architecture, module structure, data models, and implementation checklist
- **Implementation Plan**: `docs/plans/2026-02-13-artemis-rust-implementation.md` - Step-by-step implementation tasks with detailed instructions

**Java Implementation Reference:** The original Java implementation has been cloned to the local `artemis-java/` directory. If you have any questions about the implementation details, design patterns, or API contracts, please refer to the Java codebase in this directory.
