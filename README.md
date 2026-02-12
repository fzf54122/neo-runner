# neo-runner

中文 | [English](README.en.md)

`neo-runner` 是一个基于 Rust Workspace 的可扩展任务执行框架，面向“配置驱动、分层架构、插件扩展”的工程场景。📘

## ✨ 项目定位

本项目用于构建统一的任务运行平台，支持从配置加载到调度执行、结果汇报的完整链路，核心目标是：稳定、可测、可演进。

## 🧱 Workspace 结构

- `runner-cli`：命令行入口，负责参数解析与输出展示。
- `runner-core`：核心领域模型、策略、指标与统一错误。
- `runner-app`：应用编排层（`load -> plan -> execute -> report`）。
- `runner-infra`：基础设施适配（配置/进程/HTTP/文件/时间）。
- `runner-plugins`：内置插件集合（`shell/http/sql`）。
- `xtask`：仓库工程任务（格式化、检查、发布辅助等）。

## 🚀 快速开始

```bash
cargo check --workspace
cargo test --workspace
```

## 🏃 当前可运行能力

- 已打通主流程：`load -> plan -> execute -> report`
- 支持任务依赖拓扑排序（含循环依赖检测）
- 调度已支持按依赖层分批执行（batch）
- 支持 `type: shell` 任务执行
- 支持超时与重试参数解析（`default_timeout`/`timeout`、`default_retry`/`retry`）
- 执行阶段已接入 `max_concurrency` 并发上限与 `fail_fast` 行为控制
- CLI 已支持子命令：`run` / `plan` / `validate`
- CLI 已支持输出格式：`--output text|json`（当前 `plan/validate` 完整支持 JSON）
- CLI `run` 现已支持 JSON 报告（含任务级执行结果）
- 已增加最小事件流（`run_started/task_started/task_finished/run_finished`）并随 `run --output json` 输出

## 💻 运行示例

```bash
cargo run --bin runner-cli -- -f examples/demo.yaml
```

或使用子命令：

```bash
cargo run --bin runner-cli -- validate -f examples/demo.yaml
cargo run --bin runner-cli -- plan -f examples/demo.yaml
cargo run --bin runner-cli -- run -f examples/demo.yaml

# JSON 输出（便于脚本/CI 消费）
cargo run --bin runner-cli -- validate -f examples/demo.yaml --output json
cargo run --bin runner-cli -- plan -f examples/demo.yaml --output json
cargo run --bin runner-cli -- run -f examples/demo.yaml --output json
```

查看命令参数：

```bash
cargo run --bin runner-cli -- --help
```

## 📚 文档索引

- 架构说明：`docs/architecture.md`
- 配置规范：`docs/config-spec.md`
- 插件规范：`docs/plugin-spec.md`
- 路线图：`docs/roadmap.md`

## 🔐 安全与变更

- 安全策略：`SECURITY.md`
- 变更日志：`CHANGELOG.md`
