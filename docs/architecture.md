# 架构设计说明（详细版）

本文档用于指导你从零实现 `neo-runner` 的核心架构。目标不是一次性“做全”，而是在保证边界稳定的前提下，逐步把能力做深。

## 当前实现进展（阶段性）

为避免“做了很多但目标不清晰”，这里先给出当前代码与目标架构的对照：

### 已完成

1. `load + validate`：
   - YAML 配置加载（文件与字符串入口）
   - 基础字段校验（`version/name/tasks`）
   - 依赖合法性校验（不存在依赖、自依赖）
   - 策略解析（`default_timeout/timeout`、`default_retry/retry`）
2. `plan`：
   - DAG 拓扑排序
   - 依赖环检测
   - 分层批次（batch）生成
3. `execute`（MVP）：
   - `shell/http` 任务执行（`http` 为 MVP）
   - 重试、超时
   - `fail_fast` 与非 fail-fast 行为
   - 批次内并发（受 `max_concurrency` 控制）
   - 最小事件流（`run_started/task_started/task_finished/run_finished`）
4. `cli`：
   - `run / plan / validate` 子命令
   - 全局 `-f/--file` 配置入口
   - 输出模式：`--output text|json`（`run/plan/validate` 已支持 JSON）

### 未完成（离目标还有距离的部分）

1. 事件总线与可观测性：
   - 事件已进入 `RunResult`，但尚未拆分独立 eventbus 与订阅机制
2. 报告层：
   - JSON 报告已覆盖 `run/plan/validate`，并包含任务级耗时与状态码
   - 仍缺少分批次统计与更丰富聚合指标
3. 插件体系完善：
   - `http` 已有 MVP 路径，`sql` 仍是骨架，插件注册机制仍待稳定化
4. 并发策略深化：
   - 已有批次并发，尚未支持更细粒度执行控制和取消传播
5. 错误模型升级：
   - 当前仍以字符串错误为主，尚未统一到可机器解析的错误码结构

### 下一阶段建议（按优先级）

1. 事件流最小化落地：补 `task_started/task_finished/run_finished` 事件模型。
2. 报告层增强：补分批次统计、重试分布、失败聚合等字段。
3. 插件体系工程化：统一插件注册与能力声明接口（为 `sql` 和外部插件做准备）。

> 说明：后续每一阶段都采用“先补测试，再加实现，再更新 README/CHANGELOG”的节奏，避免反复返工。

---

## 1. 设计目标与约束

### 1.1 核心目标

`neo-runner` 的目标是成为一个“配置驱动”的任务执行框架，支持从配置到执行报告的完整链路：

`load -> validate -> plan -> execute -> report`

它需要同时满足：

1. **可扩展**：新任务类型可通过插件接入。
2. **可维护**：核心模型稳定，外围实现可替换。
3. **可测试**：核心逻辑不依赖 IO，易于单测。
4. **可观测**：过程事件、结果结构和错误语义统一。

### 1.2 非目标（当前阶段）

- 不追求分布式调度。
- 不引入复杂状态存储和集群高可用。
- 不做 DSL 解释器，优先 YAML 配置直驱。

---

## 2. 分层架构与 crate 职责

### 2.1 总览

- `runner-cli`：输入输出适配层（参数、输出、退出码）。
- `runner-app`：应用编排层（流程、调度、执行编排）。
- `runner-core`：领域核心层（模型、策略、错误、指标）。
- `runner-infra`：基础设施层（文件、进程、HTTP、时钟）。
- `runner-plugins`：插件实现层（按 `task.type` 分派执行）。
- `xtask`：工程自动化任务（lint/release/doc 等）。

### 2.2 依赖规则（必须遵守）

建议保持以下依赖方向：

1. `runner-cli -> runner-app`
2. `runner-app -> runner-core`
3. `runner-app -> runner-infra`
4. `runner-app -> runner-plugins`
5. `runner-plugins -> runner-core`
6. `runner-infra -> runner-core`（仅当需要共享模型/错误）

**禁止反向依赖**：`runner-core` 不能依赖 `app/infra/plugins/cli`。

这条规则会直接决定后续可测试性和重构成本。

---

## 3. 运行时主流程（建议实现顺序）

### 3.1 `load`

职责：读取配置源（本地文件为主），解析为内部配置模型。

- 输入：`Path` / 原始 YAML 文本
- 输出：`ConfigDoc`（含 `version`）
- 失败：语法错误、文件不存在、编码错误

### 3.2 `validate`

职责：业务规则校验，输出可执行的规范化配置。

- 校验内容：字段完整性、任务 ID 唯一、依赖合法、策略参数范围
- 输出：`JobSpec`（已经是可信输入）

### 3.3 `plan`

职责：把 `JobSpec` 转成执行计划。

- 处理内容：依赖图构建、拓扑排序、并发批次、matrix 展开
- 输出：`ExecutionPlan`

### 3.4 `execute`

职责：按计划调度并执行任务。

- 关注点：并发控制、重试策略、超时、取消传播、fail-fast
- 输出：`RunResult` + 过程事件流

### 3.5 `report`

职责：面向用户输出执行结果。

- 终端：简洁表格/文本
- 机器：JSON（可用于 CI 系统）
- 退出码：`0` 成功，非 `0` 失败

---

## 4. 核心领域模型（`runner-core`）

建议先稳定这些模型再扩展功能。

### 4.1 任务模型

- `TaskSpec`：单任务定义（`id`、`type`、`depends_on`、`timeout`、`retry`、`params`）
- `JobSpec`：任务集合与全局策略（并发上限、默认超时、fail-fast）

### 4.2 执行结果模型

- `TaskResult`：每个任务的最终状态与耗时
- `RunResult`：整体统计（总数、成功、失败、跳过、总耗时）

### 4.3 错误模型

建议统一 `RunnerError` 分类：

- `ConfigError`：配置加载/校验问题
- `PlanError`：依赖图/计划构建问题
- `ExecutionError`：任务执行失败
- `InternalError`：框架内部异常

错误信息应同时兼顾：人可读（CLI）和机可读（JSON code）。

---

## 5. 调度与执行策略（`runner-app`）

### 5.1 DAG 与依赖

- 任务可声明 `depends_on`
- 启动前检测环（cycle）
- 按拓扑层级生成可并发执行批次

### 5.2 并发模型

- 全局并发：`max_concurrency`
- 可选任务级并发限制（后续）
- 建议基于 `tokio` + 信号量控制

### 5.3 失败策略

- `fail_fast = true`：任一关键任务失败即停止提交新任务
- `fail_fast = false`：尽量完成可执行任务再汇总失败

### 5.4 超时与重试

- 超时应明确作用域：单次尝试超时 or 整任务超时
- 重试建议支持：最大次数、退避策略、可重试错误类型

---

## 6. 插件体系（`runner-plugins`）

### 6.1 接口建议

可定义统一执行接口（示意）：

```rust
trait TaskExecutor {
    fn kind(&self) -> &'static str;
    async fn execute(&self, task: &TaskSpec, ctx: &ExecutionContext) -> Result<TaskResult, RunnerError>;
}
```

### 6.2 分发机制

- 根据 `task.type` 查找对应插件
- 未命中时返回 `UnknownTaskType`
- 插件应避免直接依赖 CLI 输出逻辑

### 6.3 内置插件优先级

建议先做：`shell` -> `http` -> `sql`。

---

## 7. 事件与可观测性

建议事件总线最少覆盖：

- `run_started`
- `task_scheduled`
- `task_started`
- `task_retrying`
- `task_finished`
- `run_finished`

每个事件应至少包含：时间戳、任务 ID、尝试次数、状态、摘要消息。

这样后续可平滑接入日志、指标系统和审计流水。

---

## 8. 测试策略

### 8.1 单元测试

- `runner-core`：模型、策略、错误映射
- `runner-app`：计划算法、策略分支（fail-fast/重试/超时）

### 8.2 集成测试

- 在 `tests/integration_cli.rs` 读取 `examples/*.yaml`
- 校验退出码、标准输出、关键事件数量

### 8.3 回归测试

- 为每个历史 bug 补一个最小配置样例
- 放在 `tests/fixtures/` 统一管理

---

## 9. 逐步实现建议（你可以按这个节奏做）

1. 固化 `TaskSpec/JobSpec/RunResult/RunnerError`。
2. 打通 `load + validate`。
3. 实现无依赖顺序执行器。
4. 加入并发与 fail-fast。
5. 加入重试与超时。
6. 接入事件总线和 JSON 报告。
7. 引入 DAG 与 matrix。

每一步都保持“可运行 + 可测试 + 可回滚”。

---

## 10. 决策记录建议

建议在 `docs/` 下额外维护 ADR（Architecture Decision Record），例如：

- 为什么使用 workspace 分层
- 为什么选择 tokio 并发模型
- 为什么错误模型统一到 `runner-core`

这会显著降低团队后续沟通与重构成本。
