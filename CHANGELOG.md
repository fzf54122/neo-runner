# 变更日志

`neo-runner` 的重要变更将记录在本文件中。

## [Unreleased]

- 完成 workspace 分层脚手架初始化（`cli/core/app/infra/plugins/xtask`）。
- 增加基础文档、示例配置与 CI 工作流。
- `runner-infra/config_loader` 完成 `version/fail_fast/max_concurrency/depends_on/timeout/retry` 的加载与校验。
- `runner-app/scheduler` 完成基于 DAG 的拓扑计划生成与环检测。
- `runner-app/runner` 打通 `shell` 任务执行链路，支持 fail-fast、重试和超时。
- `runner-cli` 接入配置文件参数 `-f/--file`，可直接执行示例配置。
