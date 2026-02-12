# 🚀 neo-runner

<div align="center">

<img src="docs/assets/neo-runner-banner.svg" alt="neo-runner banner" width="900" />

**一个面向工程交付的 Rust 任务编排器：配置驱动、默认可靠、可观测输出。**

**简体中文** | [English](README.en.md)

[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![Binary](https://img.shields.io/badge/Binary-neo--runner-2ea043.svg)](crates/runner-cli)
[![CI](https://img.shields.io/badge/CI-fmt%20%7C%20clippy%20%7C%20test-4c9aff.svg)](.github/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-MIT-f2c94c.svg)](LICENSE)

[⚡ 快速开始](#-快速开始) • [✨ 关键能力](#-关键能力) • [📊 能力矩阵](#-能力矩阵) • [💻 示例](#-示例) • [🧪 质量保障](#-质量保障)

</div>

## 🌟 项目定位

`neo-runner` 用于把“零散脚本执行”升级为“可治理任务系统”。

- 🧭 **统一协议**：YAML 描述任务与策略，不再依赖口头约定。
- 🛡️ **默认可靠**：重试、超时、并发限制、失败策略开箱即用。
- 📈 **可观测**：`run/plan/validate` 支持 JSON 输出，便于 CI 与平台接入。

## ✨ 关键能力

- ✅ 任务类型：`shell` / `http` / `sql`（`http/sql` 为 MVP）。
- ✅ DAG 调度：拓扑排序、依赖校验、循环检测。
- ✅ 批次并发：按依赖层执行，批次内受 `max_concurrency` 控制。
- ✅ 执行策略：`default_timeout`/`timeout`、`default_retry`/`retry`、`fail_fast`。
- ✅ 报告输出：`text/json` 双模式，任务级结果含耗时与状态码。
- ✅ 报告聚合：批次统计、重试分布、失败分组。
- ✅ 事件流：`run_started/task_started/task_finished/run_finished`（支持 eventbus 订阅）。
- ✅ 执行插件注册：内置 `shell/http/sql` 通过统一执行注册表接入。
- ✅ 扩展入口：支持通过注册表注入自定义执行器（为外部插件铺路）。

## 📊 能力矩阵

| 能力域 | 当前状态 | 说明 |
|------|---------|------|
| shell 执行 | ✅ | 进程执行，支持超时/重试 |
| http 执行 | ✅ MVP | 支持方法、URL、预期状态码断言 |
| sql 执行 | ✅ MVP | SQLite 批量执行，支持 `query/sql_file` |
| 依赖调度 | ✅ | DAG 拓扑排序 + 环检测 |
| 并发控制 | ✅ | 分批次并发 + `max_concurrency` |
| 失败策略 | ✅ | `fail_fast` / 非 fail-fast |
| JSON 报告 | ✅ | `run/plan/validate` |
| 事件流 | ✅ 可订阅 | eventbus + in-memory collector |
| 插件注册机制 | ✅ 基础版 | 统一执行注册表，外部插件扩展预留 |

## 🧱 架构分层

- `runner-cli`：CLI 入口与输出层（`neo-runner`）。
- `runner-core`：领域模型（`TaskSpec`、`RunResult` 等）。
- `runner-app`：编排层（`load -> plan -> execute -> report`）。
- `runner-infra`：配置/进程/HTTP/SQL 等基础设施适配。
- `runner-plugins`：插件扩展目录（当前预留）。

## ⚡ 快速开始

```bash
cargo check --workspace
cargo test --workspace
```

查看帮助：

```bash
cargo run --bin neo-runner -- --help
```

构建发布二进制：

```bash
cargo build -p runner-cli --release
./target/release/neo-runner --help
```

本地安装（写入 `~/.cargo/bin`）：

```bash
bash scripts/install.sh
neo-runner --help
```

仓库任务（xtask）：

```bash
cargo xtask check
cargo xtask test
cargo xtask ci
cargo xtask build-release
cargo xtask doctor

# 仅打印命令，不执行
cargo xtask --dry-run ci
cargo xtask --dry-run doctor --with-check
```

## 💻 示例

基础执行（默认 `run`）：

```bash
cargo run --bin neo-runner -- -f examples/demo.yaml
```

显式子命令：

```bash
cargo run --bin neo-runner -- validate -f examples/demo.yaml
cargo run --bin neo-runner -- plan -f examples/demo.yaml
cargo run --bin neo-runner -- run -f examples/demo.yaml
```

JSON 输出（适合脚本与 CI）：

```bash
cargo run --bin neo-runner -- validate -f examples/demo.yaml --output json
cargo run --bin neo-runner -- plan -f examples/demo.yaml --output json
cargo run --bin neo-runner -- run -f examples/demo.yaml --output json
```

HTTP 并发场景：

```bash
cargo run --bin neo-runner -- run -f examples/demo-http.yaml --output json
```

SQL 批量导入场景：

```bash
cargo run --bin neo-runner -- run -f examples/demo-sql.yaml --output json
```

## 🧪 质量保障

```bash
cargo check --workspace
cargo test -p runner-infra
cargo test -p runner-app
cargo test -p runner-cli
```

CI 质量门禁：

- `fmt`：格式检查
- `clippy`：静态检查
- `test`：单元测试 + 集成测试

## 📚 文档索引

- 架构设计：`docs/architecture.md`
- 配置规范：`docs/config-spec.md`
- 插件规范：`docs/plugin-spec.md`
- 路线规划：`docs/roadmap.md`

## 🗺️ 路线图摘要

- 📌 报告增强：分批次统计、重试分布、失败聚合
- 📌 事件系统：从最小事件流升级到可订阅 eventbus
- 📌 插件工程化：统一注册机制与能力声明

## 🔐 安全与版本

- 安全策略：`SECURITY.md`
- 变更日志：`CHANGELOG.md`

## 🤝 贡献

欢迎 Issue / PR，共建任务编排能力。建议流程：

1. 阅读 `docs/architecture.md` 与 `docs/config-spec.md`
2. 先补测试，再实现，再更新文档
3. 保持阶段性提交（小步快跑）

---

> 如果你准备把任务体系接入生产，建议先从 `examples/demo.yaml`、`examples/demo-http.yaml`、`examples/demo-sql.yaml` 跑通一轮，再逐步迁移业务任务。🔥
