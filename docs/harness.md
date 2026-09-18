# 跨 Harness 安装

同一份二进制，三份说明书。用户不需要这份仓库的本地代码。不要为 Claude 单独做 MCP。

官网：<https://fzf54122.github.io/neo-runner/>

## 一条命令

```bash
curl -fsSL https://raw.githubusercontent.com/fzf54122/neo-runner/main/scripts/install.sh | bash
```

这会：

- 从 GitHub Releases 安装 `neo-runner` 到 `~/.local/bin`（可用 `NEO_RUNNER_PREFIX` 改）
- 尽量把该目录写进 shell `PATH`
- 本机有 `claude` CLI 时：`claude plugin marketplace add fzf54122/neo-runner`，再 `claude plugin marketplace update neo-runner`，然后 `claude plugin install neo-runner@neo-runner --scope user`
- 写入用户级 skill：`~/.claude/skills/neo-runner`、`~/.agents/skills/neo-runner`；有 Codex 时再写 `~/.codex/skills/neo-runner`

只装二进制：

```bash
NEO_RUNNER_SKIP_PLUGINS=1 curl -fsSL https://raw.githubusercontent.com/fzf54122/neo-runner/main/scripts/install.sh | bash
```

没有预编译包时：

```bash
cargo install --git https://github.com/fzf54122/neo-runner --tag v0.2.0 --bin neo-runner
```

确认：

```bash
neo-runner --version
```

卸载全局安装（不删项目里的 `.agents/loop.yaml`）：

```bash
curl -fsSL https://raw.githubusercontent.com/fzf54122/neo-runner/main/scripts/uninstall.sh | bash
```

这会：

- 删除 `~/.local/bin/neo-runner`（以及 `~/.cargo/bin/neo-runner` 残留）
- 去掉 `install.sh` 写入的 `# neo-runner` PATH 行
- 删除用户级 skill：`~/.claude/skills/neo-runner`、`~/.agents/skills/neo-runner`、`~/.codex/skills/neo-runner`
- 卸载 Claude plugin / marketplace，并清掉 cache / data 残留

只看会删什么：`bash scripts/uninstall.sh --dry-run`。本机若装了 `.deb`，默认只提示；真正清包用 `--purge-deb`。

## 项目循环文件

全局安装不写当前目录。进仓库后：

```bash
neo-runner init
```

已存在 `.agents/loop.yaml` 时拒绝，除非 `--force`。`init` 按仓库根文件写入常见门禁，也可用 `--preset rust|go|python|node|generic` 覆盖。不够就改 YAML。没有该文件时 hook 会跳过。

同时写项目级 skill：

```bash
neo-runner init --skill
```

## Claude Code

`install.sh` 优先走非交互 CLI。没有 `claude` 时，在对话框里输入：

```text
/plugin marketplace add fzf54122/neo-runner
/plugin marketplace update neo-runner
/plugin install neo-runner@neo-runner
```

Stop hook 只随 marketplace plugin 加载。仅有 `~/.claude/skills/` 兜底 skill 时没有 hook。

## Codex / OpenCode / 其它 SKILL.md 宿主

用户级文件由 `install.sh` 写入。项目级：

```bash
neo-runner init --skill
```

把 `neo-runner` 放进 `PATH` 即可。

## DeepSeek Harness (DSH)

DSH 把 skill 当插件说明书，执行仍走 CLI：

```bash
neo-runner run -f .agents/loop.yaml --output json
```

不要把 neo-runner 做成 MCP server，除非客户端没有 Bash。
