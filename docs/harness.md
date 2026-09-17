# 跨 Harness 安装

同一份二进制，三份说明书。不要为 Claude 单独做 MCP。

## 二进制

```bash
# 本仓库
bash scripts/install.sh

# GitHub
cargo install --git https://github.com/fzf54122/neo-runner --bin neo-runner
```

确认：

```bash
neo-runner --version
```

## Claude Code

```text
/plugin marketplace add fzf54122/neo-runner
/plugin install neo-runner
```

这会装上 `skills/neo-runner/SKILL.md` 和 Stop hook。项目里还要有 `.agents/loop.yaml`，否则 hook 会跳过。

项目级只放 skill、不走 marketplace 时：

```bash
mkdir -p .claude/skills/neo-runner
cp skills/neo-runner/SKILL.md .claude/skills/neo-runner/SKILL.md
```

## Codex / OpenCode / 其它 SKILL.md 宿主

本仓库已经带了 `.agents/skills/neo-runner/SKILL.md`。把 `neo-runner` 放进 `PATH` 即可。

拷到另一个项目时：

```bash
mkdir -p .agents/skills/neo-runner
cp /path/to/neo-runner/.agents/skills/neo-runner/SKILL.md .agents/skills/neo-runner/SKILL.md
```

## DeepSeek Harness (DSH)

DSH 把 skill 当插件说明书，执行仍走 CLI：

```bash
neo-runner run -f .agents/loop.yaml --output json
```

把 `skills/neo-runner/SKILL.md` 拷进当前 DSH 的 plugin/skill 目录，并给该 skill 打 `dsh-plugin` topic（若你单独拆仓库发布）。

不要把 neo-runner 做成 MCP server，除非客户端没有 Bash。
