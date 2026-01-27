---
title: "chatwoot-cli: generate Rust CLI from Chatwoot OpenAPI"
status: closed
priority: 1
issue-type: task
created-at: "\"\\\"2026-01-27T21:51:37.494111+07:00\\\"\""
closed-at: "2026-01-27T21:58:11.312926+07:00"
close-reason: "Implemented OpenAPI-based CLI + generator, packaged arm64 tarball, created GitHub release v0.1.0. Validation: cargo build --release; ./target/release/chatwoot list --json."
---

Scope: generator + runtime, install script, formula, release asset. Files: Cargo.toml, src/*, tools/*, schemas/*, scripts/*, Formula/*, README.md, LICENSE. Acceptance: cargo build --release; ./target/release/chatwoot list --json works.
