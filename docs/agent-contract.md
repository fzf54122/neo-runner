# Agent 契约

`neo-runner` 对外只保证这一件事：Agent 不能在没有证据时声称做完。

## 命令

```bash
neo-runner run -f .agents/loop.yaml --output json
```

- 退出码 `0`：`ok` 为 `true`
- 退出码 `1`：任务失败（JSON 仍会打到 stdout）
- 退出码 `2`：配置缺失或无法加载

`validate` / `plan` 只检查 YAML，不是完工信号。

## JSON 字段（`run --output json`）

Agent 只应依赖这些字段：

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| `ok` | bool | 唯一绿灯。与 `success` 同值，保留 `success` 仅为兼容旧脚本 |
| `failed_tasks` | string[] | 仍失败的 task id |
| `evidence` | object[] | 失败证据：`task`、`exit_code`、`excerpt` |
| `duration_ms` | number | 整次 run 的墙钟时间 |

`excerpt` 优先取 stderr，其次 stdout，再退回错误信息，最长 2000 字符（超出保留尾部）。

完整 payload 仍包含 `tasks` / `events` / `batches` 等调试字段，Agent 不必读。

## 循环文件

项目把 Definition of Done 写在 `.agents/loop.yaml`。用 `neo-runner init` 从模板写入（模板也在 `examples/agent-loop.yaml`）。

## Hook

Claude Code Stop hook（`hooks/stop-gate.sh`）：

- 没有 `.agents/loop.yaml`：跳过（exit 0）
- 有 loop 文件但没有二进制：拦截（exit 1）
- 跑红：拦截，并把 JSON 打到 stdout
