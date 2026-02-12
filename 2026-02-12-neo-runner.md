---
title: "🚀 neo-runner：把 shell/http/sql 任务统一到一个可靠执行器"
date: 2026-02-12
categories: [Rust, 任务编排, 工程化]
tags: [neo-runner, shell, http, sql, DAG, 并发, 重试, 超时, JSON 报告]
pin: false
description: "🎯 一个配置驱动的 Rust 任务运行器：内置 DAG 调度、并发批次、超时重试、结构化 JSON 报告与最小事件流。"
---

<div align="center">

## ⚙️ neo-runner

**把“脚本能跑”升级成“任务稳定可控地跑”。**

`shell` • `http` • `sql` 统一协议，统一调度，统一报告。

</div>

---

## 🧩 背景：为什么做这个项目？

很多团队的自动化任务都经历过同一条路径：

- 一开始是几段 shell，能用就行
- 后来任务越来越多，依赖关系靠约定
- 出问题时日志分散，定位慢、复盘难
- 想接 CI 或观测系统时，缺少结构化输出

于是你会发现，真正的痛点不是“执行命令”，而是：

> ❗ 如何让任务在复杂场景下 **稳定、可追踪、可演进** 地运行。

`neo-runner` 就是为这个问题而生：

✅ 一份 YAML 配置跑通 `load -> plan -> execute -> report`，并把执行细节以 JSON 交给上层系统消费。

---

## ⚖️ 方案对比：脚本方案 vs neo-runner

| 维度 | 传统脚本方案 | neo-runner |
|------|--------------|------------|
| 任务定义 | 零散 shell/python 脚本，分布不统一 | 统一 YAML 配置协议 |
| 依赖关系 | 依赖人工约定，容易漏执行顺序 | 内置 DAG 拓扑排序 + 依赖校验 |
| 并发控制 | 需手写并发逻辑，风险高 | 批次并发 + `max_concurrency` |
| 重试与超时 | 各脚本各写，策略不一致 | `default_retry/timeout` + task 覆盖 |
| 失败策略 | 通常只看退出码，行为不透明 | `fail_fast` 明确控制 |
| 输出结果 | 文本日志为主，机器消费困难 | `text/json` 双输出，结构化报告 |
| 可观测性 | 缺少统一事件语义 | 最小事件流（run/task started/finished） |
| 扩展任务类型 | 每种任务单独造轮子 | `shell/http/sql` 统一执行入口 |
| 回归测试 | 脚本链路难做系统化验证 | 配置加载/调度/CLI 集成测试完整 |
| 团队协作 | 知识依赖个人经验 | 协议化 + 分层架构，认知成本更低 |

> 简单说：脚本方案“起步快”，但规模上来后维护成本高；`neo-runner` 让任务系统从“可运行”升级到“可治理”。

---

## 🎯 设计目标与边界

### 目标

- 🧱 **配置驱动**：统一协议，减少脚本散落
- 🔁 **默认可靠**：重试、超时、并发、fail-fast
- 📊 **可观测**：任务级报告 + 事件流
- 🧰 **可扩展**：任务类型逐步插件化

### 当前边界

- 不做分布式调度
- 不做复杂状态存储
- 先聚焦单机可靠执行与工程化交付

---

## 🏗️ 当前实现到哪了？

下面是当前版本的能力快照：

| 模块 | 当前状态 | 说明 |
|------|----------|------|
| 配置加载 | ✅ | 支持 `version/job/tasks` 与策略解析校验 |
| 调度计划 | ✅ | DAG 拓扑排序 + 依赖环检测 + 分批次（batch） |
| 执行引擎 | ✅ | 支持批次内并发、重试、超时、fail-fast |
| 任务类型 | ✅ MVP | `shell/http/sql` |
| 输出报告 | ✅ | `text/json`，run 含任务级结果 |
| 事件流 | ✅ 最小版 | `run_started/task_started/task_finished/run_finished` |

---

## ✨ 三类任务，统一执行

### 1) 🖥️ shell 任务

- 适合本地命令、构建、测试、脚本链路
- 支持超时与重试

### 2) 🌐 http 任务（并发友好）

- 适合健康检查、服务探活、API 预检
- 支持状态码断言
- 依赖批次调度可并发执行多 HTTP 任务

### 3) 🗃️ sql 任务（批量导入）

- 适合初始化、数据导入、批量修复脚本
- 支持 `query` 和 `sql_file`
- 当前以 SQLite 路径为 MVP 先跑通场景

---

## 🚀 运行示例

### 基础任务

```bash
cargo run --bin neo-runner -- run -f examples/demo.yaml --output json
```

### HTTP 任务（并发场景）

```bash
cargo run --bin neo-runner -- run -f examples/demo-http.yaml --output json
```

### SQL 批量导入

```bash
cargo run --bin neo-runner -- run -f examples/demo-sql.yaml --output json
```

---

## 📦 报告长什么样？

`run --output json` 当前输出关键信息：

- 全局：`success` / `total` / `failed`
- 任务级：`id` / `success` / `attempts` / `error`
- 细节：`duration_ms` / `exit_code` / `status_code`
- 事件：`run_started` / `task_started` / `task_finished` / `run_finished`

这意味着：

✅ 人可以看执行结果，系统也可以直接消费和聚合。

---

## 🪨 过程中的关键坑位

### 坑 1：CLI 参数兼容性

引入子命令后，`-f` 默认行为曾失效。

修复：把 `-f/--file` 变成全局参数，兼容默认 run 与显式子命令。

### 坑 2：JSON 输出被运行日志污染

如果执行日志混在 stdout，会破坏 JSON 解析。

修复：确保结构化结果稳定输出，并在测试中按 JSON 结果断言。

### 坑 3：配置相对路径语义

`sql_file` / `sqlite://` 相对路径应以配置文件为基准，而不是进程 cwd。

修复：在 `load_yaml(path)` 阶段做路径归一化。

---

## 🧭 下一步计划

1. 📈 报告增强：分批次统计、重试分布、失败聚合
2. 📡 事件体系升级：从最小事件流到可订阅 eventbus
3. 🧩 插件工程化：统一注册机制，为外部插件铺路

---

## 💬 结语

`neo-runner` 的方向很明确：

> 不是再造一个“脚本启动器”，而是提供一个能逐步演进为工程基础设施的任务执行层。

如果你的团队也在维护越来越复杂的自动化任务链，欢迎直接从 `examples/` 跑起来试一轮。🔥
