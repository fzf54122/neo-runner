# 🚀 neo-runner

<div align="center">

<img src="docs/assets/neo-runner-banner.svg" alt="neo-runner banner" width="900" />

**Agent 说「做完了」不算。`neo-runner` 绿灯才算。**

**简体中文** | [English](README.en.md)

[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![Binary](https://img.shields.io/badge/Binary-neo--runner-2ea043.svg)](crates/runner-cli)
[![Release](https://img.shields.io/github/v/release/fzf54122/neo-runner.svg)](https://github.com/fzf54122/neo-runner/releases)
[![CI](https://img.shields.io/badge/CI-fmt%20%7C%20clippy%20%7C%20test-4c9aff.svg)](.github/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-MIT-f2c94c.svg)](LICENSE)
[![Stars](https://img.shields.io/github/stars/fzf54122/neo-runner?style=social)](https://github.com/fzf54122/neo-runner/stargazers)

[官网](https://fzf54122.github.io/neo-runner/) • [⚡ 安装](#-安装) • [🔌 Claude Code](#-在-claude-code-里用) • [✨ 关键能力](#-关键能力) • [⭐ Star 历史](#-star-历史)

</div>

## 🌟 项目定位

`neo-runner` 是给编码 Agent 的完工门禁：YAML 描述循环，JSON 给出证据，只有退出码 0 才允许声称完成。

底层仍是原来的 Rust 任务编排器（DAG、重试、超时、并发），只是默认用法改成了 **二进制 + Skill + Hook**，不是 MCP。

- 🧭 **循环即契约**：把 fmt/test/lint 写进 `.agents/loop.yaml`。
- 🛡️ **红灯不准撒谎**：`ok: false` 时 Agent 不得说「已完成」。
- 📈 **JSON 证据**：`failed_tasks` + `evidence[].excerpt` 告诉下一步改什么。

官网：[https://fzf54122.github.io/neo-runner/](https://fzf54122.github.io/neo-runner/)

契约说明见 [docs/agent-contract.md](docs/agent-contract.md)，安装见 [docs/harness.md](docs/harness.md)。

## ⚡ 安装

用户不需要这份仓库的本地代码。一条命令装好二进制和 Claude / Codex 插件：

```bash
curl -fsSL https://raw.githubusercontent.com/fzf54122/neo-runner/main/scripts/install.sh | bash
neo-runner --version
```

进具体仓库后再写循环文件：

```bash
neo-runner init
```

把 `.agents/loop.yaml` 里的 `echo fmt-ok` / `echo test-ok` 换成这个项目真正的门禁。没有该文件时 hook 会 skip。

只装二进制：`NEO_RUNNER_SKIP_PLUGINS=1` 再跑上面的 `curl | bash`。没有预编译包时：

```bash
cargo install --git https://github.com/fzf54122/neo-runner --tag v0.2.0 --bin neo-runner
```

手动取附件见 [GitHub Releases](https://github.com/fzf54122/neo-runner/releases)。

确认：

```bash
neo-runner run -f .agents/loop.yaml --output json
```

卸载全局安装（不删项目里的 `.agents/loop.yaml`）：

```bash
curl -fsSL https://raw.githubusercontent.com/fzf54122/neo-runner/main/scripts/uninstall.sh | bash
```

## 🔌 在 Claude Code 里用

三件东西，不是 MCP：

| 层 | 作用 |
| --- | --- |
| 二进制 `neo-runner` | 真正跑 `.agents/loop.yaml` |
| Skill | 告诉模型完工前必须跑哪条命令、怎么读 JSON |
| Stop hook | 项目里有 `.agents/loop.yaml` 时，会话结束前强制再跑一遍；红灯就拦 |

`install.sh` 会调用 `claude plugin marketplace add fzf54122/neo-runner`、`claude plugin marketplace update neo-runner`，再 `claude plugin install neo-runner@neo-runner --scope user`。没有 `claude` CLI 时，在对话框里输入：

```text
/plugin marketplace add fzf54122/neo-runner
/plugin marketplace update neo-runner
/plugin install neo-runner@neo-runner
```

然后对 Claude 说：

```text
用 neo-runner 验收一下，别口头说通过。
```

模型会跑：

```bash
neo-runner run -f .agents/loop.yaml --output json
```

- 退出码 `0` 且 `ok: true`：才允许说做完
- 退出码 `1`：红灯。JSON 仍在 stdout，读 `failed_tasks` 和 `evidence[].excerpt` 再改
- 退出码 `2`：配置缺失或 YAML 加载失败

`/plugin` 能看到 `neo-runner` 已启用即加载成功。plugin 装一次是用户级的，每个项目只要有自己的 `.agents/loop.yaml`。

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
- ✅ 错误模型：配置/调度/执行均支持结构化错误码输出。
- ✅ Agent 契约：`run --output json` 输出 `ok` / `failed_tasks` / `evidence` / `duration_ms`，失败退出码为 1。
- ✅ Claude Code plugin：`SKILL.md` + Stop hook（有 `.agents/loop.yaml` 才拦截）。

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
| Agent 契约 | ✅ | `ok` / `failed_tasks` / `evidence`，失败退出 1 |
| Skill + Hook | ✅ | Claude plugin + Codex `SKILL.md` |
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

从本仓库源码安装（开发者）：

```bash
cargo install --path crates/runner-cli --bin neo-runner
neo-runner --help
```

仓库任务（xtask）：

```bash
cargo xtask check
cargo xtask test
cargo xtask ci
cargo xtask build-release
cargo xtask release
cargo xtask doctor

# 仅打印命令，不执行
cargo xtask --dry-run ci
cargo xtask --dry-run doctor --with-check
```

生成 shell 自动补全：

```bash
# 生成 zsh 补全脚本
cargo run --bin neo-runner -- completion zsh > _neo-runner

# 生成 bash 补全脚本
cargo run --bin neo-runner -- completion bash > neo-runner.bash
```

`cargo xtask release` 会输出 Debian 包（`dist/neo-runner_*_amd64.deb`）和对应 `sha256` 校验文件。
安装 `.deb` 后会自动安装补全文件：`bash` / `zsh` / `fish`。

GitHub `Release` 工作流支持：

- 推送 `main` 时，若提交信息包含 `feat:` 或 `fix:` 前缀，会自动打包并发布预发布版本。
- 推送 `v*` tag 时，自动发布正式版本。
- 发布附件包含：`.deb`、Linux 原生二进制、Windows `.exe`（含 zip 与校验文件）。
- Release 页面会按 `feat:` / `fix:` 分类显示本次变更。
- 发布前会使用 `upx -9` 压缩可执行文件以减小体积。
- Linux 发布二进制基于 `x86_64-unknown-linux-musl` 构建，避免宿主机 `glibc` 版本不兼容。

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

Agent 循环（完工门禁）：

```bash
cargo run --bin neo-runner -- run -f examples/agent-loop.yaml --output json
```

JSON 输出（适合脚本、CI 与 Agent）：

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

综合场景（`shell + http + sql`）：

```bash
cargo run --bin neo-runner -- run -f examples/demo-all.yaml --output json
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
- Agent 契约：`docs/agent-contract.md`
- 跨 Harness 安装：`docs/harness.md`
- 路线规划：`docs/roadmap.md`

## 🗺️ 路线图摘要

- 📌 报告增强：分批次统计、重试分布、失败聚合
- 📌 事件系统：从最小事件流升级到可订阅 eventbus
- 📌 插件工程化：统一注册机制与能力声明

## ⭐ Star 历史

[![Star History Chart](https://api.star-history.com/chart?repos=fzf54122/neo-runner&type=Date)](https://www.star-history.com/#fzf54122/neo-runner&Date)

## 🔐 安全与版本

- 官网：<https://fzf54122.github.io/neo-runner/>
- 安全策略：`SECURITY.md`
- 变更日志：`CHANGELOG.md`

## 🤝 贡献

欢迎 Issue / PR，共建任务编排能力。建议流程：

1. 阅读 `docs/architecture.md` 与 `docs/config-spec.md`
2. 先补测试，再实现，再更新文档
3. 保持阶段性提交（小步快跑）

---

> 如果你准备把任务体系接入生产，建议先从 `examples/demo.yaml`、`examples/demo-http.yaml`、`examples/demo-sql.yaml` 跑通一轮，再逐步迁移业务任务。🔥
