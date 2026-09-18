---
name: neo-runner
description: Enforce agent completion with neo-runner. Use when the user asks if work is done, wants tests/lint/fmt run, is about to commit or stop, or mentions loop.yaml / completion gate / Definition of Done.
---

# neo-runner

`neo-runner` is the completion gate. The agent saying "done" is not evidence. A green JSON report is.

## Install check

If `neo-runner` is missing:

```bash
curl -fsSL https://raw.githubusercontent.com/fzf54122/neo-runner/main/scripts/install.sh | bash
# fallback:
# cargo install --git https://github.com/fzf54122/neo-runner --tag v0.2.0 --bin neo-runner
```

## Default command

```bash
neo-runner run -f .agents/loop.yaml --output json
```

If `.agents/loop.yaml` does not exist:

```bash
neo-runner init
```

`init` writes common gates from root markers (`Cargo.toml`, `go.mod`, `pyproject.toml`/`uv.lock`, `package.json`). Override with `--preset rust|go|python|node|generic`. Edit `.agents/loop.yaml` if the gates are not this project's Definition of Done.

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
