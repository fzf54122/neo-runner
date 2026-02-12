# 配置规范（详细版）

本文档定义 `neo-runner` 的 YAML 配置协议（Config Protocol）。你在实现时可以先覆盖最小子集，再逐步补齐高级能力。

---

## 1. 设计目标

配置协议应满足：

1. **可读**：人类易写、易审查。
2. **可校验**：错误尽早暴露，报错可定位。
3. **可演进**：通过 `version` 支持未来升级。
4. **可扩展**：插件参数对框架透明，核心字段稳定。

---

## 2. 顶层结构

```yaml
version: 1

job:
  name: demo
  description: "optional description"
  fail_fast: true
  max_concurrency: 4
  default_timeout: "30s"
  default_retry:
    max_attempts: 1
    backoff: "fixed"
    delay: "1s"

  env:
    APP_ENV: dev

  matrix:
    os: [linux, macos]
    rust: [stable, beta]

  tasks:
    - id: lint
      type: shell
      cmd: "cargo clippy --workspace -- -D warnings"

    - id: test
      type: shell
      depends_on: [lint]
      cmd: "cargo test --workspace"
```

---

## 3. 字段定义

### 3.1 `version`（必填）

- 类型：`integer`
- 当前值：`1`
- 作用：协议版本控制

不兼容变更必须提升 `version`。

### 3.2 `job`（必填）

`job` 是一次执行单元的配置容器。

字段：

- `name`（必填，string）：任务组标识。
- `description`（可选，string）：任务组说明。
- `fail_fast`（可选，bool，默认 `true`）：失败是否快速中断。
- `max_concurrency`（可选，int，默认 `1`）：全局并发上限。
- `default_timeout`（可选，duration）：任务默认超时。
- `default_retry`（可选，object）：任务默认重试策略。
- `env`（可选，map<string,string>）：全局环境变量。
- `matrix`（可选，map<string,list<string>>>）：矩阵展开维度。
- `tasks`（必填，array<Task>）：任务列表，至少 1 项。

---

## 4. Task 定义

### 4.1 通用字段

每个 task 至少包含：

- `id`（必填，string）：任务唯一标识，`[a-zA-Z0-9_-]+`。
- `type`（必填，string）：插件类型，如 `shell`、`http`。

可选字段：

- `name`（string）：展示名称。
- `depends_on`（array<string>）：依赖任务 ID。
- `if`（string）：条件表达式（后续可实现）。
- `timeout`（duration）：覆盖默认超时。
- `retry`（object）：覆盖默认重试。
- `env`（map<string,string>）：任务级环境变量。
- `continue_on_error`（bool，默认 `false`）：失败是否不中断。
- `params`（map<string,any>）：插件扩展参数。

### 4.2 插件专属字段建议

#### `type: shell`

- `cmd`（必填，string）：执行命令。
- `cwd`（可选，string）：工作目录。
- `shell`（可选，string）：解释器，如 `bash`/`sh`。

#### `type: http`

- `method`（必填，string）：`GET/POST/...`
- `url`（必填，string）：目标地址。
- `headers`（可选，map<string,string>）
- `body`（可选，string/object）
- `expected_status`（可选，int 或 array<int>）

#### `type: sql`（预留）

- `dsn`（必填，string）
- `query`（必填，string）
- `tx`（可选，bool）

---

## 5. 默认值与覆盖规则

配置优先级（高 -> 低）：

1. 任务级字段（`task.timeout/retry/env`）
2. `job` 默认字段（`default_timeout/default_retry/env`）
3. 系统内置默认值

示例：

- `task.timeout` 存在时覆盖 `job.default_timeout`
- `task.env` 与 `job.env` 合并，同名键以 `task.env` 为准

---

## 6. Matrix 展开规则

### 6.1 目标

把一个逻辑任务扩展为多个物理任务实例。

### 6.2 规则建议

1. `job.matrix` 使用笛卡尔积展开。
2. 展开后每个实例应拥有唯一运行 ID（如 `task@os=linux,rust=stable`）。
3. `cmd/url/params` 可引用变量（如 `${os}`）。

### 6.3 示例

```yaml
matrix:
  os: [linux, macos]
  rust: [stable, beta]
```

将展开为 4 组上下文。

---

## 7. 校验规则（实现时建议最先落地）

### 7.1 结构校验

- 必填字段存在
- 类型正确
- 不允许未知顶层字段（可配置为 warn/deny）

### 7.2 语义校验

- `task.id` 不重复
- `depends_on` 指向存在任务
- 依赖图无环
- `max_concurrency >= 1`
- `retry.max_attempts >= 1`

### 7.3 安全校验（可选）

- 拒绝明显危险命令（按策略）
- 对 URL/schema 做白名单约束

---

## 8. 错误输出规范

建议错误对象结构：

```yaml
code: CONFIG_VALIDATION_ERROR
message: "task 'build' depends_on unknown task 'lintx'"
path: "job.tasks[1].depends_on[0]"
hint: "Did you mean 'lint'?"
```

其中 `path` 字段非常关键，能显著提升可用性。

---

## 9. 版本演进策略

### 9.1 原则

- 向后兼容增强：保持 `version` 不变。
- 破坏性变更：提升 `version` 并提供迁移文档。

### 9.2 实现建议

在 `runner-infra/config_loader` 建立：

1. `parse_raw(versioned_doc)`
2. `upgrade_to_latest_if_needed(...)`
3. `validate_strict(...)`
4. `to_domain(JobSpec)`

避免在业务执行层处理兼容逻辑。

---

## 10. 最小实现子集（建议你先做这个）

如果你想快速跑通 MVP，先实现以下字段即可：

- 顶层：`version`、`job.name`、`job.tasks`
- task：`id`、`type`、`depends_on`、`cmd`（shell）
- 策略：`fail_fast`、`max_concurrency`、`timeout`、`retry.max_attempts`

然后再按优先级补：`env` -> `http` -> `matrix` -> `sql`。

---

## 11. 示例：完整配置

```yaml
version: 1
job:
  name: "ci-pipeline"
  fail_fast: true
  max_concurrency: 3
  default_timeout: "60s"
  default_retry:
    max_attempts: 2
    backoff: "fixed"
    delay: "2s"

  env:
    RUST_LOG: info

  matrix:
    os: [linux, macos]

  tasks:
    - id: fmt
      type: shell
      cmd: "cargo fmt --all --check"

    - id: clippy
      type: shell
      depends_on: [fmt]
      cmd: "cargo clippy --workspace --all-targets -- -D warnings"

    - id: unit-test
      type: shell
      depends_on: [clippy]
      timeout: "120s"
      retry:
        max_attempts: 1
      cmd: "cargo test --workspace"

    - id: health
      type: http
      method: GET
      url: "https://example.com/health"
      expected_status: [200, 204]
```

---
