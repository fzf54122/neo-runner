---
name: neo-runner
description: Enforce agent completion with neo-runner. Use when the user asks if work is done, wants tests/lint/fmt run, is about to commit or stop, or mentions loop.yaml / completion gate / Definition of Done.
---

# neo-runner

`neo-runner` is the completion gate. The agent saying "done" is not evidence. A green JSON report is.

## Install check

If `neo-runner` is missing:

```bash
cargo install --path crates/runner-cli --bin neo-runner
# or from GitHub:
# cargo install --git https://github.com/fzf54122/neo-runner --bin neo-runner
```

## Default command

```bash
neo-runner run -f .agents/loop.yaml --output json
```

If `.agents/loop.yaml` does not exist, copy `examples/agent-loop.yaml` and replace the shell commands with this project's real gates (`cargo test`, `uv run pytest`, `pnpm test`, …).

Fallback when the project has not adopted `.agents/loop.yaml` yet:

```bash
neo-runner run -f examples/agent-loop.yaml --output json
```

## How to read the JSON

Trust only these fields:

| field | meaning |
| --- | --- |
| `ok` | `true` is the only green signal |
| `failed_tasks` | task ids still red |
| `evidence[].excerpt` | last stdout/stderr (or error) for a failed task |
| `duration_ms` | wall time of the whole run |

Process exit code is part of the contract: `0` only when `ok` is `true`. Non-zero means not done.

Example red report:

```json
{
  "ok": false,
  "failed_tasks": ["test"],
  "evidence": [
    { "task": "test", "exit_code": 1, "excerpt": "FAILED tests/test_auth.py::test_login" }
  ],
  "duration_ms": 18420
}
```

## Hard rules

1. Before claiming a task is complete, run neo-runner and wait for `ok: true`.
2. If `ok` is `false`, do not say tests passed / work is done / ready to commit.
3. Use `failed_tasks` and `evidence[].excerpt` to decide what to fix next. Then rerun.
4. Do not invent a passing result. Do not skip the loop because the change "looks small".
5. `validate` / `plan` are for inspecting YAML. They are not completion.

## Project loop shape

```yaml
version: 1
job:
  name: agent-loop
  fail_fast: true
  tasks:
    - id: fmt
      type: shell
      cmd: "cargo fmt --all -- --check"
    - id: test
      type: shell
      depends_on: [fmt]
      cmd: "cargo test --workspace"
```

Keep commands deterministic. Prefer check/test over format-in-place unless the user asked to write files.
