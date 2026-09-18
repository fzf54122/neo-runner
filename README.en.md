# 🚀 neo-runner

<div align="center">

<img src="docs/assets/neo-runner-banner.svg" alt="neo-runner banner" width="900" />

**The agent saying "done" is not done. `neo-runner` green is done.**

[中文](README.md) | **English**

[![Rust](https://img.shields.io/badge/Rust-2021-orange.svg)](https://www.rust-lang.org/)
[![Binary](https://img.shields.io/badge/Binary-neo--runner-2ea043.svg)](crates/runner-cli)
[![Release](https://img.shields.io/github/v/release/fzf54122/neo-runner.svg)](https://github.com/fzf54122/neo-runner/releases)
[![CI](https://img.shields.io/badge/CI-fmt%20%7C%20clippy%20%7C%20test-4c9aff.svg)](.github/workflows/ci.yml)
[![License](https://img.shields.io/badge/License-MIT-f2c94c.svg)](LICENSE)
[![Stars](https://img.shields.io/github/stars/fzf54122/neo-runner?style=social)](https://github.com/fzf54122/neo-runner/stargazers)

[Website](https://fzf54122.github.io/neo-runner/) • [Install](#-install) • [Claude Code](#-use-with-claude-code) • [Star History](#-star-history)

</div>

## 🌟 Positioning

`neo-runner` is a completion gate for coding agents: YAML describes the loop, JSON is the evidence, and exit code 0 is the only signal that work is done.

The engine is still the original Rust orchestrator (DAG, retries, timeouts, concurrency). The product is **binary + Skill + Hook**, not an MCP server.

- 🧭 **Loop as contract**: put fmt/test/lint in `.agents/loop.yaml`.
- 🛡️ **Red means not done**: if `ok` is `false`, the agent must not claim completion.
- 📈 **JSON evidence**: `failed_tasks` + `evidence[].excerpt` tell it what to fix next.

Website: [https://fzf54122.github.io/neo-runner/](https://fzf54122.github.io/neo-runner/)

See [docs/agent-contract.md](docs/agent-contract.md) and [docs/harness.md](docs/harness.md).

## ⚡ Install

Users do not need a local clone. One command installs the binary and Claude / Codex plugins:

```bash
curl -fsSL https://raw.githubusercontent.com/fzf54122/neo-runner/main/scripts/install.sh | bash
neo-runner --version
```

Then, in a project:

```bash
neo-runner init
```

Replace `echo fmt-ok` / `echo test-ok` in `.agents/loop.yaml` with this project's real gates. If the file is missing, the hook skips.

Binary only: set `NEO_RUNNER_SKIP_PLUGINS=1` before the curl. If there is no prebuilt asset:

```bash
cargo install --git https://github.com/fzf54122/neo-runner --tag v0.2.0 --bin neo-runner
```

Manual downloads: [GitHub Releases](https://github.com/fzf54122/neo-runner/releases).

Verify:

```bash
neo-runner run -f .agents/loop.yaml --output json
```

## 🔌 Use with Claude Code

Three pieces, not MCP:

| Layer | Role |
| --- | --- |
| Binary `neo-runner` | Actually runs `.agents/loop.yaml` |
| Skill | Tells the model which command to run and how to read JSON |
| Stop hook | If the project has `.agents/loop.yaml`, force a rerun before the session ends; red blocks completion |

`install.sh` runs `claude plugin marketplace add fzf54122/neo-runner` and `claude plugin install fzf54122@neo-runner --scope user`. Without the `claude` CLI, type this in the session:

```text
/plugin marketplace add fzf54122/neo-runner
/plugin install fzf54122@neo-runner
```

Then tell Claude:

```text
Verify with neo-runner. Do not claim it passed verbally.
```

The model runs:

```bash
neo-runner run -f .agents/loop.yaml --output json
```

- Exit `0` and `ok: true`: the only green signal
- Exit `1`: red. JSON is still on stdout; read `failed_tasks` and `evidence[].excerpt`
- Exit `2`: missing config or YAML failed to load

`/plugin` should list `neo-runner` as enabled. The plugin is user-level; each project only needs its own `.agents/loop.yaml`.

## ✨ Capabilities

- ✅ Task types: `shell` / `http` / `sql` (`http/sql` are MVP).
- ✅ DAG scheduling: topological sort + cycle detection.
- ✅ Batch concurrency: dependency-layer execution with `max_concurrency`.
- ✅ Execution policies: `default_timeout`/`timeout`, `default_retry`/`retry`, `fail_fast`.
- ✅ Reporting: `text/json`, task-level details (duration, exit/status codes).
- ✅ Event stream: `run_started/task_started/task_finished/run_finished`.
- ✅ Error model: structured error codes across load/plan/execute paths.
- ✅ Agent contract: `run --output json` emits `ok` / `failed_tasks` / `evidence` / `duration_ms`; failed runs exit 1.
- ✅ Claude Code plugin: `SKILL.md` + Stop hook (intercepts only when `.agents/loop.yaml` exists).

## 📊 Capability Matrix

| Area | Status | Notes |
|------|--------|-------|
| shell execution | ✅ | process execution with retry/timeout |
| http execution | ✅ MVP | method/url + expected status checks |
| sql execution | ✅ MVP | SQLite batch via `query/sql_file` |
| DAG scheduler | ✅ | topological batches + cycle detection |
| Concurrency control | ✅ | batch-level parallelism + cap |
| Failure strategy | ✅ | fail-fast / non-fail-fast |
| JSON reporting | ✅ | `run/plan/validate` |
| Agent contract | ✅ | `ok` / `failed_tasks` / `evidence`; failed runs exit 1 |
| Skill + Hook | ✅ | Claude plugin + Codex `SKILL.md` |
| Event bus | ✅ | subscribable event bus + collector |
| External plugins | 🚧 | lifecycle spec drafted, dynamic loading pending |

## 🧱 Architecture Layers

- `runner-cli`: CLI entry and output surface (`neo-runner`).
- `runner-core`: domain models (`TaskSpec`, `RunResult`, etc.).
- `runner-app`: orchestration pipeline (`load -> plan -> execute -> report`).
- `runner-infra`: config/process/http/sql/fs/clock adapters.
- `runner-plugins`: plugin extension directory (reserved for expansion).

## ⚡ Quick Start

```bash
cargo check --workspace
cargo test --workspace
```

CLI help:

```bash
cargo run --bin neo-runner -- --help
```

Build release binary:

```bash
cargo build -p runner-cli --release
./target/release/neo-runner --help
```

Install from this source tree (developers):

```bash
cargo install --path crates/runner-cli --bin neo-runner
neo-runner --help
```

Repository tasks (`xtask`):

```bash
cargo xtask check
cargo xtask test
cargo xtask ci
cargo xtask build-release
cargo xtask release
cargo xtask doctor
```

Generate shell completions:

```bash
# Generate zsh completion script
cargo run --bin neo-runner -- completion zsh > _neo-runner

# Generate bash completion script
cargo run --bin neo-runner -- completion bash > neo-runner.bash
```

`cargo xtask release` produces a Debian package (`dist/neo-runner_*_amd64.deb`) and matching `sha256` file.
Installing the `.deb` automatically installs completion scripts for `bash` / `zsh` / `fish`.

GitHub `Release` workflow supports:

- On `main` pushes, if commit messages include `feat:` or `fix:`, it auto-packages and publishes a prerelease.
- On `v*` tag pushes, it publishes a stable release.
- Release assets include `.deb`, Linux raw binary, and Windows `.exe` (with zip/checksums).
- The Release page groups changes by `feat:` and `fix:` commit prefixes.
- Executables are compressed with `upx -9` before packaging to reduce artifact size.
- Linux binaries are built with `x86_64-unknown-linux-musl` to avoid host `glibc` version mismatch.

## 💻 Examples

Default run:

```bash
cargo run --bin neo-runner -- -f examples/demo.yaml
```

Subcommands:

```bash
cargo run --bin neo-runner -- validate -f examples/demo.yaml
cargo run --bin neo-runner -- plan -f examples/demo.yaml
cargo run --bin neo-runner -- run -f examples/demo.yaml
```

Agent loop (completion gate):

```bash
cargo run --bin neo-runner -- run -f examples/agent-loop.yaml --output json
```

JSON output:

```bash
cargo run --bin neo-runner -- run -f examples/demo.yaml --output json
```

HTTP concurrency scenario:

```bash
cargo run --bin neo-runner -- run -f examples/demo-http.yaml --output json
```

SQL batch import scenario:

```bash
cargo run --bin neo-runner -- run -f examples/demo-sql.yaml --output json
```

All-in-one scenario (`shell + http + sql`):

```bash
cargo run --bin neo-runner -- run -f examples/demo-all.yaml --output json
```

## 🧪 Quality

```bash
cargo check --workspace
cargo test -p runner-infra
cargo test -p runner-app
cargo test -p runner-cli
```

## 📚 Documents

- Architecture: `docs/architecture.md`
- Config spec: `docs/config-spec.md`
- Plugin spec: `docs/plugin-spec.md`
- Agent contract: `docs/agent-contract.md`
- Cross-harness install: `docs/harness.md`
- Roadmap: `docs/roadmap.md`

## 🗺️ Roadmap Snapshot

- 📌 Reporting enhancement: batch statistics, retry distribution, failure grouping.
- 📌 Event evolution: from minimal lifecycle events to richer subscriptions.
- 📌 Plugin engineering: capability declaration and external plugin runtime path.

## ⭐ Star History

[![Star History Chart](https://api.star-history.com/chart?repos=fzf54122/neo-runner&type=Date)](https://www.star-history.com/#fzf54122/neo-runner&Date)

## 🔐 Security & Versioning

- Website: <https://fzf54122.github.io/neo-runner/>
- Security policy: `SECURITY.md`
- Changelog: `CHANGELOG.md`

## 🤝 Contributing

Contributions via Issues and PRs are welcome. Suggested workflow:

1. Read `docs/architecture.md` and `docs/config-spec.md` first.
2. Add/adjust tests before implementation, then update docs.
3. Keep phase-based commits small and reviewable.

---

> If you are preparing production onboarding, start with `examples/demo.yaml`, `examples/demo-http.yaml`, `examples/demo-sql.yaml`, and `examples/demo-all.yaml`, then migrate business jobs incrementally.
