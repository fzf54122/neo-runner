# neo-runner

[中文](README.md) | English

`neo-runner` is a Rust workspace project for building a configurable and extensible job runner. It is designed around clear layering, plugin-based extension, and production-friendly engineering practices.

## Project Focus

`neo-runner` provides an end-to-end execution flow:

`load -> plan -> execute -> report`

The architecture prioritizes maintainability, testability, and long-term evolution.

## Workspace Crates

- `runner-cli`: CLI entrypoint, argument parsing, and output rendering.
- `runner-core`: domain model, policy, metrics, and unified errors.
- `runner-app`: orchestration layer and execution pipeline.
- `runner-infra`: infrastructure adapters (config/process/http/fs/clock).
- `runner-plugins`: built-in plugins (`shell/http/sql`).
- `xtask`: repository-level automation tasks.

## Quick Start

```bash
cargo check --workspace
cargo test --workspace
```

## Documents

- `docs/architecture.md`
- `docs/config-spec.md`
- `docs/plugin-spec.md`
- `docs/roadmap.md`
