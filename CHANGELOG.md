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
- `runner-infra/http` 增加 HTTP 请求抽象入口（MVP），并接入 `runner-app` 的 `http` 任务执行路径。
- `config_loader` 支持解析 `http` 任务字段（`method/url/expected_status`）。
- 新增 `http` 任务执行单元测试与配置解析测试。
- `RunResult` 的任务级报告新增 `duration_ms/exit_code/status_code` 字段，便于追踪执行细节。
- `run --output json` 已输出耗时和状态细节，CLI 集成测试已覆盖关键字段。
- `runner-infra/sql` 增加 SQLite 批量执行能力（支持 `query` 与 `sql_file`）。
- `runner-app` 接入 `type: sql` 任务执行链路，可用于批量导入场景。
- `config_loader` 增加 SQL 字段解析（`dsn/query/sql_file`）与对应测试。
- 增加 SQL 示例配置与批处理脚本（`examples/demo-sql.yaml`、`examples/demo-batch.sql`）。
- CLI 二进制统一为 `neo-runner`（支持 `cargo run --bin neo-runner` 与 `cargo install --bin neo-runner`）。
- 重写 `README.md` 为更聚焦的工程化风格，强调能力矩阵与上手路径。
- 新增博客草稿：`2026-02-12-neo-runner.md`（参考 relihttp 文风）。
- `runner-app/eventbus` 增加可订阅事件总线与内存采集器，执行事件通过总线发布。
- `runner-app/executor` 增加统一执行注册表，内置 `shell/http/sql` 通过注册机制接入。
- `RunResult` 新增报告聚合字段：`batches/retry_distribution/failure_groups`。
- `runner-cli` JSON 输出同步包含批次统计、重试分布与失败聚合信息。
- `xtask` 增加工程命令：`fmt/clippy/test/check/ci/build-release`，支持 `--dry-run`。
- `runner-app/runner` 增加 `run_job_with_registry` 扩展入口，支持注入自定义执行注册表。
- 新增 HTTP 并发行为测试（对比并发与串行耗时），验证并发执行路径生效。
