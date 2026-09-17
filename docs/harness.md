# 跨 Harness 安装

同一份二进制，三份说明书。用户不需要这份仓库的本地代码。不要为 Claude 单独做 MCP。

官网：<https://fzf54122.github.io/neo-runner/>

## 二进制

Release 工作流会发布 Linux / Windows 附件，直接下载：

```bash
curl -fsSL https://raw.githubusercontent.com/fzf54122/neo-runner/main/scripts/install.sh | bash
```

或打开 [Releases](https://github.com/fzf54122/neo-runner/releases/tag/v0.2.0) 取：

- `neo-runner-linux-x86_64.tar.gz`
- `neo-runner_*_amd64.deb`
- `neo-runner.exe`

没有预编译包时：

```bash
cargo install --git https://github.com/fzf54122/neo-runner --tag v0.2.0 --bin neo-runner
```

确认：

```bash
neo-runner --version
```

## 项目循环文件

```bash
mkdir -p .agents
curl -fsSL https://raw.githubusercontent.com/fzf54122/neo-runner/main/examples/agent-loop.yaml \
  -o .agents/loop.yaml
```

把命令换成这个项目的真实门禁。没有 `.agents/loop.yaml` 时 hook 会跳过。

## Claude Code

```text
/plugin marketplace add fzf54122/neo-runner
/plugin install neo-runner
```

这会装上 `skills/neo-runner/SKILL.md` 和 Stop hook。项目里还要有 `.agents/loop.yaml`，否则 hook 会跳过。

只给当前项目一份 skill、不走 marketplace 时：

```bash
mkdir -p .claude/skills/neo-runner
curl -fsSL https://raw.githubusercontent.com/fzf54122/neo-runner/main/skills/neo-runner/SKILL.md \
  -o .claude/skills/neo-runner/SKILL.md
```

## Codex / OpenCode / 其它 SKILL.md 宿主

```bash
mkdir -p .agents/skills/neo-runner
curl -fsSL https://raw.githubusercontent.com/fzf54122/neo-runner/main/.agents/skills/neo-runner/SKILL.md \
  -o .agents/skills/neo-runner/SKILL.md
```

把 `neo-runner` 放进 `PATH` 即可。

## DeepSeek Harness (DSH)

DSH 把 skill 当插件说明书，执行仍走 CLI：

```bash
neo-runner run -f .agents/loop.yaml --output json
```

把 `skills/neo-runner/SKILL.md` 拷进当前 DSH 的 plugin/skill 目录。不要把 neo-runner 做成 MCP server，除非客户端没有 Bash。
