---
name: neo-runner
description: Enforce agent completion with neo-runner. Use when the user asks if work is done, wants tests/lint/fmt run, is about to commit or stop, or mentions loop.yaml / completion gate / Definition of Done.
---

# neo-runner

Canonical skill for Codex / OpenCode / other `SKILL.md` hosts. Claude Code loads `skills/neo-runner/SKILL.md` from the plugin.

`neo-runner` is the completion gate. The agent saying "done" is not evidence. A green JSON report is.

## Default command

```bash
neo-runner run -f .agents/loop.yaml --output json
```

If the loop file is missing:

```bash
neo-runner init
```

Then replace the `echo` placeholders with this project's real gates.

## How to read the JSON

| field | meaning |
| --- | --- |
| `ok` | `true` is the only green signal |
| `failed_tasks` | task ids still red |
| `evidence[].excerpt` | last stdout/stderr (or error) for a failed task |
| `duration_ms` | wall time of the whole run |

Exit code `0` only when `ok` is `true`.

## Hard rules

1. Before claiming a task is complete, run neo-runner and wait for `ok: true`.
2. If `ok` is `false`, do not say tests passed / work is done / ready to commit.
3. Use `failed_tasks` and `evidence[].excerpt` to decide what to fix next. Then rerun.
4. Do not invent a passing result.
5. `validate` / `plan` inspect YAML. They are not completion.
