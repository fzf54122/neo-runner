# 变更日志

`neo-runner` 的重要变更将记录在本文件中。

## [Unreleased]

- 完成 workspace 分层脚手架初始化（`cli/core/app/infra/plugins/xtask`）。
- 增加基础文档、示例配置与 CI 工作流。
- `runner-infra/config_loader` 完成 `version/fail_fast/max_concurrency/depends_on/timeout/retry` 的加载与校验。
- `runner-app/scheduler` 完成基于 DAG 的拓扑计划生成与环检测。
- `runner-app/runner` 打通 `shell` 任务执行链路，支持 fail-fast、重试和超时。
- `runner-cli` 接入配置文件参数 `-f/--file`，可直接执行示例配置。
- `runner-cli` 增加 `run/plan/validate` 子命令，支持按阶段运行与排查。
- 新增 `runner-app/scheduler` 单元测试与 `runner-cli` 集成测试，降低回归风险。
- `runner-app/scheduler` 增加分层批次计划（batch）输出，支持后续并发执行扩展。
- `runner-app/runner` 执行阶段接入 `max_concurrency` 并发限制，按批次并发执行任务。
- 新增 `runner-app/runner` 失败策略测试（`fail_fast=true/false`）与批次调度测试。
- `runner-cli` 增加 `--output text|json` 输出模式，`plan/validate` 支持结构化 JSON 输出。
- 新增 CLI JSON 输出集成测试，保障自动化场景稳定性。
- `runner-core` 引入任务级执行结果模型（`TaskRunResult`），`RunResult` 增加 `failed/tasks` 字段。
- `runner-cli run --output json` 现可输出任务级报告（任务 id、成功状态、尝试次数、错误信息）。
- `runner-app` 增加最小事件流（`run_started/task_started/task_finished/run_finished`），并纳入 `RunResult`。
- `runner-cli` 的 `run --output json` 现包含事件数组，便于后续可观测性接入。
