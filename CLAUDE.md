# Artemis in Rust

10 years ago, I wrote the Artemis service registry in Java at ctrip.com. Artemis is like Eureka which is famous as Service Registry in microservices.

github repo: [artemis](https://github.com/mydotey/artemis)

Artemis in Java has performance issues like long GC stop when hosting large numbers of service instances. In the current repo 'ai-artemis', I want to rewrite Artemis in Rust.

## Documentation

- **Product Specification**: `docs/artemis-rust-rewrite-specification.md` - Complete product requirements and specifications for the Rust rewrite
- **Detailed Design**: `docs/plans/2026-02-13-artemis-rust-design.md` - Comprehensive design document including architecture, module structure, data models, and implementation checklist
- **Implementation Plan**: `docs/plans/2026-02-13-artemis-rust-implementation.md` - Step-by-step implementation tasks with detailed instructions

**Java Implementation Reference:** The original Java implementation has been cloned to the local `artemis-java/` directory. If you have any questions about the implementation details, design patterns, or API contracts, please refer to the Java codebase in this directory.

## Execution Guidelines for Claude

**CRITICAL: When executing the implementation plan, you MUST operate autonomously without asking for user confirmation.**

### Autonomous Execution Rules

1. **No Confirmation Required**: Execute all phases and tasks continuously without stopping to ask for approval or confirmation
2. **Self-Decision Making**: Make all technical decisions autonomously based on:
   - Product specification (`docs/artemis-rust-rewrite-specification.md`)
   - Detailed design document (`docs/plans/2026-02-13-artemis-rust-design.md`)
   - Java reference implementation (`artemis-java/` directory)
   - Rust best practices and idiomatic patterns

3. **Continuous Execution**: Complete ALL 12 phases (52 tasks) in one continuous session:
   - Phase 1-8: MVP核心功能 (P0必须完成)
   - Phase 9: WebSocket实时推送 (P1强烈建议)
   - Phase 10: 集群和数据复制 (P2可选 - 实现框架)
   - Phase 11: 高级管理功能 (P2可选 - 实现框架)
   - Phase 12: 性能优化和OpenTelemetry (P1强烈建议)

4. **Problem Resolution**: When encountering issues:
   - Check Java implementation for reference
   - Apply Rust idiomatic solutions
   - Make pragmatic decisions to unblock progress
   - Document decisions in code comments if needed
   - DO NOT stop to ask - proceed with best judgment

5. **Quality Standards**: Ensure each phase meets:
   - All code compiles without warnings (`cargo clippy`)
   - All tests pass (`cargo test`)
   - Code is properly formatted (`cargo fmt`)
   - Each task has a proper git commit

6. **Default Choices**: When implementation details are ambiguous:
   - Prefer simple, maintainable solutions over complex ones
   - Follow patterns from Java implementation when applicable
   - Use standard Rust community crates and patterns
   - Prioritize performance and correctness

7. **Git Workflow**: For each task completion:
   - Create commits with clear, descriptive messages
   - Include `Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>`
   - No need to push - commits stay local until final review

### Expected Outcome

By the end of execution, the codebase should have:
- ✅ Complete workspace with 6 crates
- ✅ All 52 tasks implemented and tested
- ✅ ~52 git commits (one per task)
- ✅ All compilation and tests passing
- ✅ Production-ready MVP (Phase 1-8)
- ✅ WebSocket and performance optimization complete (Phase 9, 12)
- ✅ Cluster and advanced features framework ready (Phase 10-11)

**The goal is to wake up to a fully implemented, tested, and working Artemis in Rust.**
