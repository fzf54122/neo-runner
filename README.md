# 🚀 neo-runner

<div align="center">

**一个面向生产场景的 Rust 任务编排器：配置驱动、分层架构、默认可靠。**

**简体中文** | [English](README.en.md)

[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![Workspace](https://img.shields.io/badge/Cargo-Workspace-blue.svg)](Cargo.toml)
[![CLI](https://img.shields.io/badge/Binary-neo--runner-green.svg)](crates/runner-cli)
[![License](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

[⚡ 快速开始](#-快速开始) • [✨ 核心能力](#-核心能力) • [💻 运行示例](#-运行示例) • [📚 文档索引](#-文档索引)

</div>

## 🌟 为什么是 neo-runner？

在自动化任务系统里，最常见的问题不是“能不能跑”，而是“**是否稳定可控地跑**”。`neo-runner` 聚焦三个目标：

- **配置驱动**：统一 YAML 协议，降低脚本散落和心智负担。
- **默认可靠**：内置依赖排序、重试、超时、并发控制和 fail-fast。
- **工程友好**：结构化 JSON 报告 + 最小事件流，便于 CI/观测系统接入。

## 🧱 架构分层

- `runner-cli`：命令入口与输出格式化（text/json）。
- `runner-core`：领域模型与执行结果结构。
- `runner-app`：主流程编排（`load -> plan -> execute -> report`）。
- `runner-infra`：配置加载、进程执行、HTTP、SQL 等基础设施适配。
- `runner-plugins`：插件扩展目录（当前以内置任务类型为主）。

## ✨ 核心能力

- 支持任务类型：`shell` / `http` / `sql`（`http/sql` 为 MVP）。
- 支持 DAG 依赖拓扑排序与分层批次（batch）调度。
- 支持批次内并发（`max_concurrency`）与失败策略（`fail_fast`）。
- 支持超时与重试策略（`default_timeout`/`timeout`，`default_retry`/`retry`）。
- 支持结构化 JSON 报告（任务级状态、耗时、状态码、事件流）。
- 支持最小事件流：`run_started` / `task_started` / `task_finished` / `run_finished`。

## ⚡ 快速开始

```bash
cargo check --workspace
cargo test --workspace
```

查看 CLI 帮助：

```bash
cargo run --bin neo-runner -- --help
```

## 💻 运行示例

基础执行（等价于 `run`）：

```bash
cargo run --bin neo-runner -- -f examples/demo.yaml
```

显式子命令：

```bash
cargo run --bin neo-runner -- validate -f examples/demo.yaml
cargo run --bin neo-runner -- plan -f examples/demo.yaml
cargo run --bin neo-runner -- run -f examples/demo.yaml
```

JSON 输出（适合 CI/脚本消费）：

```bash
cargo run --bin neo-runner -- validate -f examples/demo.yaml --output json
cargo run --bin neo-runner -- plan -f examples/demo.yaml --output json
cargo run --bin neo-runner -- run -f examples/demo.yaml --output json
```

HTTP 并发场景示例：

```bash
cargo run --bin neo-runner -- run -f examples/demo-http.yaml --output json
```

SQL 批量导入场景示例：

```bash
cargo run --bin neo-runner -- run -f examples/demo-sql.yaml --output json
```

## 📚 文档索引

- 架构说明：`docs/architecture.md`
- 配置规范：`docs/config-spec.md`
- 插件规范：`docs/plugin-spec.md`
- 路线图：`docs/roadmap.md`

## 🔐 安全与版本

- 安全策略：`SECURITY.md`
- 变更日志：`CHANGELOG.md`

---

> 如果你希望快速落地到生产，建议先从 `examples/demo.yaml` 与 `examples/demo-http.yaml` 开始，确认环境和配置规范后再接入你的业务任务。✨
