---
title: "🚀 neo-runner：把 shell/http/sql 统一成一个可靠的任务执行器"
date: 2026-02-12
categories: [Rust, 任务编排, 工程化]
tags: [neo-runner, shell, http, sql, DAG, 并发, 重试, 超时]
pin: false
description: "🎯 一个配置驱动的 Rust 任务运行器：支持 DAG 依赖、并发批次、超时重试、结构化 JSON 报告与最小事件流。"
---

## 背景：为什么要做 neo-runner

在很多项目里，自动化任务往往从几段 shell 脚本开始，最后演变为：

- 脚本散落在多个仓库和目录
- 任务依赖关系难以维护
- 出错后只能看零散日志，排查效率低
- 想接入 CI 或观测系统时缺少结构化输出

`neo-runner` 的目标很直接：

> 用一个统一的 YAML 协议，稳定地跑 `shell/http/sql` 任务，并给出可消费的执行报告。

---

## 设计原则：先可用，再可演进

`neo-runner` 没有一开始就追求“大而全”，而是沿着一条很实用的路径推进：

1. 先打通 `load -> plan -> execute -> report` 主流程
2. 再补齐默认可靠能力（重试、超时、并发、fail-fast）
3. 最后增强可观测性和输出结构

这样做的好处是：

- 每个阶段都可运行、可测试、可回滚
- 不会因为过度设计而长期停留在“规划阶段”

---

## 核心能力（当前）

### 1) 配置驱动 + 严格校验

支持版本化 YAML，包含：

- `version`
- `job` 级策略（`fail_fast/max_concurrency/default_timeout/default_retry`）
- `task` 级策略覆盖（`timeout/retry`）
- 依赖字段 `depends_on`

并在加载阶段做基础语义校验：

- 任务 ID 唯一
- 依赖存在且不可自依赖
- 参数范围合法（如 `max_attempts >= 1`）

### 2) DAG 计划 + 批次并发

任务先做拓扑排序，再分批次执行：

- 同一批次任务互不依赖，可并发执行
- 批次间严格按依赖顺序推进
- 并发数受 `max_concurrency` 控制

### 3) 多任务类型执行（MVP）

- `shell`：进程执行，支持超时/重试
- `http`：请求执行，支持状态码断言
- `sql`：SQLite 批量执行（`query` 或 `sql_file`）

### 4) 结构化报告 + 最小事件流

`run --output json` 提供机器可读结果：

- 全局：`success/total/failed`
- 任务级：`id/success/attempts/error/duration_ms/exit_code/status_code`
- 事件：`run_started/task_started/task_finished/run_finished`

---

## 示例：一条命令跑起来

```bash
cargo run --bin neo-runner -- run -f examples/demo.yaml --output json
```

HTTP 示例：

```bash
cargo run --bin neo-runner -- run -f examples/demo-http.yaml --output json
```

SQL 批量导入示例：

```bash
cargo run --bin neo-runner -- run -f examples/demo-sql.yaml --output json
```

---

## 过程中踩过的几个坑

### 坑 1：CLI 参数兼容性

在引入子命令后，`-f` 参数的默认用法一度失效。

修复策略：

- 把 `-f/--file` 设为全局参数
- 同时兼容“默认 run”与“显式子命令 run/plan/validate”

### 坑 2：JSON 输出被运行日志污染

任务执行期间的标准输出可能影响 JSON 解析。

修复策略：

- 运行结果统一在最后输出 JSON
- 测试中按“最后一行 JSON”做断言

### 坑 3：配置相对路径语义

`sql_file` 与 `sqlite://` 的相对路径需要相对配置文件路径解析，而不是进程 cwd。

修复策略：

- `load_yaml(path)` 阶段统一做基路径解析，避免运行时歧义

---

## 适合哪些场景

- CI 任务编排（lint/test/check/release）
- 定时巡检（HTTP health 检查 + SQL 校验）
- 数据导入/初始化（SQL 批量脚本）
- 需要结构化执行报告并接入上层系统的自动化场景

---

## 下一步路线

1. 报告层继续增强：分批次统计、重试分布、失败聚合
2. 事件系统工程化：从最小事件流升级到可订阅 eventbus
3. 插件机制稳定化：为外部插件接入留出统一入口

---

## 结语

`neo-runner` 不是“又一个脚本工具”，而是一个可以逐步演进为工程基础设施的执行层。

如果你也在维护越来越复杂的自动化任务链，欢迎从 `examples/` 开始试跑，看看它是否适合你的场景。💪
