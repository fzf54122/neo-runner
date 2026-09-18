// @Time    : 2026/9/18 12:00
// @Author  : fzf
// @FileName: copy.js
// @Software: Claude Code

export const REPO_URL = 'https://github.com/fzf54122/neo-runner'
export const INSTALL_CMD =
  'curl -fsSL https://raw.githubusercontent.com/fzf54122/neo-runner/main/scripts/install.sh | bash'
export const UNINSTALL_CMD =
  'curl -fsSL https://raw.githubusercontent.com/fzf54122/neo-runner/main/scripts/uninstall.sh | bash'
export const RUN_CMD = 'neo-runner run -f .agents/loop.yaml --output json'
export const INIT_CMD = 'neo-runner init'
export const CARGO_CMD =
  'cargo install --git https://github.com/fzf54122/neo-runner --tag v0.2.0 --bin neo-runner'
export const SKIP_PLUGINS_CMD = `NEO_RUNNER_SKIP_PLUGINS=1 ${INSTALL_CMD}`

export const copy = {
  zh: {
    logo: 'neo-runner',
    docs: '文档',
    github: 'GitHub',
    getStarted: 'Get Started',
    heroTitle: '从终端验收。',
    heroSub: 'Agent 说做完了不算。绿灯才算。',
    heroKicker: 'Fast. Simple. Local-first.',
    installHint: '一条命令装好二进制、Skill 和 Hook。',
    builtTitle: 'Built for developers.',
    oneCommandTitle: 'One command. Everything you need.',
    oneCommandDesc: '不写当前目录。进仓库后再 `neo-runner init`。',
    gateTitle: 'YAML 写循环。JSON 给证据。',
    gateLead: 'Agent 只应依赖这些字段。`validate` / `plan` 只检查 YAML，不是完工信号。',
    stackTitle: '三件东西，不是 MCP。',
    stackLead: '同一份二进制，Skill 告诉模型怎么读，Hook 在会话结束前再跑一遍。',
    runTitle: '验收只认这一条命令。',
    runLead: '退出码 0 且 `ok: true` 才允许说做完。红灯时 JSON 仍在 stdout。',
    yamlLabel: 'loop.yaml',
    jsonLabel: 'stdout',
    moreDocs: '完整契约 →',
    footer: '© 2026 neo-runner · MIT License',
    theme: { light: '浅色模式', dark: '深色模式', auto: '自动模式' },
    docsTitle: '文档',
    docsSearch: '搜索文档…',
    copyLabel: '复制',
    copiedLabel: '已复制',
    starLabel: 'Star',
    versionLabel: 'v0.2.0',
    traits: [
      { title: 'Fast', desc: 'Rust 二进制。DAG、重试、超时、并发都在本机跑完。' },
      { title: 'Simple', desc: 'YAML 写循环，JSON 给证据。退出码 0 才是绿灯。' },
      { title: 'Native', desc: '二进制 + Skill + Hook，不是 MCP。' },
    ],
    stack: [
      { title: '二进制', desc: 'neo-runner 真正跑 .agents/loop.yaml。' },
      { title: 'Skill', desc: '告诉模型完工前必须跑哪条命令、怎么读 JSON。' },
      { title: 'Hook', desc: '项目里有 loop 文件时，会话结束前强制再跑一遍；红灯就拦。' },
    ],
    exits: [
      { code: '0', title: '绿灯', desc: 'ok: true。才允许说做完。' },
      { code: '1', title: '红灯', desc: 'JSON 仍在 stdout。读 failed_tasks 和 evidence[].excerpt。' },
      { code: '2', title: '配置', desc: '循环文件缺失，或 YAML 加载失败。' },
    ],
    fields: [
      { name: 'ok', desc: '唯一绿灯。' },
      { name: 'failed_tasks', desc: '仍失败的 task id。' },
      { name: 'evidence', desc: 'task / exit_code / excerpt。' },
      { name: 'duration_ms', desc: '整次 run 的墙钟时间。' },
    ],
    installs: [
      { title: 'curl', desc: '推荐。从 GitHub Releases 装二进制，并尽量写入 PATH。' },
      { title: 'cargo', desc: '没有预编译包时，从源码安装指定 tag。' },
      { title: 'init', desc: '进仓库后写入 .agents/loop.yaml。没有该文件时 hook 会 skip。' },
    ],
  },
  en: {
    logo: 'neo-runner',
    docs: 'Docs',
    github: 'GitHub',
    getStarted: 'Get Started',
    heroTitle: 'Done from the terminal.',
    heroSub: 'Agents don’t get to claim done. The gate does.',
    heroKicker: 'Fast. Simple. Local-first.',
    installHint: 'One command installs the binary, Skill, and Hook.',
    builtTitle: 'Built for developers.',
    oneCommandTitle: 'One command. Everything you need.',
    oneCommandDesc: 'Nothing is written to the current directory. Run `neo-runner init` in the repo.',
    gateTitle: 'YAML describes the loop. JSON is the evidence.',
    gateLead: 'Agents should depend only on these fields. `validate` / `plan` check YAML; they are not a done signal.',
    stackTitle: 'Three pieces. Not MCP.',
    stackLead: 'One binary. The Skill tells the model how to read it. The Hook runs it again before the session ends.',
    runTitle: 'One command is the gate.',
    runLead: 'Exit 0 and `ok: true` are the only green light. On red, JSON is still on stdout.',
    yamlLabel: 'loop.yaml',
    jsonLabel: 'stdout',
    moreDocs: 'Full contract →',
    footer: '© 2026 neo-runner · MIT License',
    theme: { light: 'Light', dark: 'Dark', auto: 'Auto' },
    docsTitle: 'Docs',
    docsSearch: 'Search docs…',
    copyLabel: 'Copy',
    copiedLabel: 'Copied',
    starLabel: 'Star',
    versionLabel: 'v0.2.0',
    traits: [
      { title: 'Fast', desc: 'A Rust binary. DAG, retries, timeouts, and concurrency stay local.' },
      { title: 'Simple', desc: 'YAML describes the loop. JSON is the evidence. Exit 0 is the only green light.' },
      { title: 'Native', desc: 'Binary + Skill + Hook. Not MCP.' },
    ],
    stack: [
      { title: 'Binary', desc: 'neo-runner actually runs .agents/loop.yaml.' },
      { title: 'Skill', desc: 'Tells the model which command to run and how to read the JSON.' },
      { title: 'Hook', desc: 'If the loop file exists, the session cannot end on a red run.' },
    ],
    exits: [
      { code: '0', title: 'Green', desc: 'ok: true. Done is allowed.' },
      { code: '1', title: 'Red', desc: 'JSON is still on stdout. Read failed_tasks and evidence[].excerpt.' },
      { code: '2', title: 'Config', desc: 'Missing loop file, or YAML failed to load.' },
    ],
    fields: [
      { name: 'ok', desc: 'The only green light.' },
      { name: 'failed_tasks', desc: 'Task ids that still failed.' },
      { name: 'evidence', desc: 'task / exit_code / excerpt.' },
      { name: 'duration_ms', desc: 'Wall-clock time of the run.' },
    ],
    installs: [
      { title: 'curl', desc: 'Recommended. Installs the GitHub Releases binary and tries to update PATH.' },
      { title: 'cargo', desc: 'Fallback when no prebuilt package exists. Pins a release tag.' },
      { title: 'init', desc: 'Writes .agents/loop.yaml in the repo. The hook skips if that file is missing.' },
    ],
  },
}

export const docNav = {
  zh: [
    { id: 'start', title: '快速开始' },
    { id: 'contract', title: 'Agent 契约' },
    { id: 'harness', title: '跨 Harness 安装' },
  ],
  en: [
    { id: 'start', title: 'Quick start' },
    { id: 'contract', title: 'Agent contract' },
    { id: 'harness', title: 'Harness install' },
  ],
}

export const docs = {
  zh: {
    start: {
      title: '快速开始',
      content: `\`neo-runner\` 是给编码 Agent 的完工门禁：YAML 描述循环，JSON 给出证据，只有退出码 0 才允许声称完成。

产品是 **二进制 + Skill + Hook**，不是 MCP。

## 安装

\`\`\`bash
${INSTALL_CMD}
neo-runner --version
\`\`\`

进具体仓库后再写循环文件：

\`\`\`bash
${INIT_CMD}
\`\`\`

把 \`.agents/loop.yaml\` 里的占位命令换成这个项目真正的门禁。没有该文件时 hook 会 skip。

只装二进制：

\`\`\`bash
${SKIP_PLUGINS_CMD}
\`\`\`

没有预编译包时：

\`\`\`bash
${CARGO_CMD}
\`\`\`

卸载全局安装（不删项目里的 \`.agents/loop.yaml\`）：

\`\`\`bash
${UNINSTALL_CMD}
\`\`\`

## 验收

\`\`\`bash
${RUN_CMD}
\`\`\`

- 退出码 \`0\` 且 \`ok: true\`：才允许说做完
- 退出码 \`1\`：红灯。JSON 仍在 stdout，读 \`failed_tasks\` 和 \`evidence[].excerpt\` 再改
- 退出码 \`2\`：配置缺失或 YAML 加载失败
`,
    },
    contract: {
      title: 'Agent 契约',
      content: `\`neo-runner\` 对外只保证这一件事：Agent 不能在没有证据时声称做完。

## 命令

\`\`\`bash
${RUN_CMD}
\`\`\`

- 退出码 \`0\`：\`ok\` 为 \`true\`
- 退出码 \`1\`：任务失败（JSON 仍会打到 stdout）
- 退出码 \`2\`：配置缺失或无法加载

\`validate\` / \`plan\` 只检查 YAML，不是完工信号。

## JSON 字段

Agent 只应依赖这些字段：

| 字段 | 类型 | 含义 |
| --- | --- | --- |
| \`ok\` | bool | 唯一绿灯。与 \`success\` 同值，保留 \`success\` 仅为兼容旧脚本 |
| \`failed_tasks\` | string[] | 仍失败的 task id |
| \`evidence\` | object[] | 失败证据：\`task\`、\`exit_code\`、\`excerpt\` |
| \`duration_ms\` | number | 整次 run 的墙钟时间 |

\`excerpt\` 优先取 stderr，其次 stdout，再退回错误信息，最长 2000 字符（超出保留尾部）。

## 循环文件

项目把 Definition of Done 写在 \`.agents/loop.yaml\`。用 \`neo-runner init\` 从模板写入。

## Hook

Claude Code Stop hook：

- 没有 \`.agents/loop.yaml\`：跳过（exit 0）
- 有 loop 文件但没有二进制：拦截（exit 1）
- 跑红：拦截，并把 JSON 打到 stdout
`,
    },
    harness: {
      title: '跨 Harness 安装',
      content: `同一份二进制，三份说明书。用户不需要这份仓库的本地代码。不要为 Claude 单独做 MCP。

## 一条命令

\`\`\`bash
${INSTALL_CMD}
\`\`\`

这会：

- 从 GitHub Releases 安装 \`neo-runner\` 到 \`~/.local/bin\`
- 尽量把该目录写进 shell \`PATH\`
- 本机有 \`claude\` CLI 时安装 marketplace plugin
- 写入用户级 skill

只装二进制：

\`\`\`bash
${SKIP_PLUGINS_CMD}
\`\`\`

卸载全局安装（不删项目里的 \`.agents/loop.yaml\`）：

\`\`\`bash
${UNINSTALL_CMD}
\`\`\`

## 项目循环文件

全局安装不写当前目录。进仓库后：

\`\`\`bash
${INIT_CMD}
\`\`\`

已存在 \`.agents/loop.yaml\` 时拒绝，除非 \`--force\`。同时写项目级 skill：

\`\`\`bash
neo-runner init --skill
\`\`\`

## Claude Code

没有 \`claude\` CLI 时，在对话框里输入：

\`\`\`text
/plugin marketplace add fzf54122/neo-runner
/plugin marketplace update neo-runner
/plugin install neo-runner@neo-runner
\`\`\`

Stop hook 只随 marketplace plugin 加载。

不要把 neo-runner 做成 MCP server，除非客户端没有 Bash。
`,
    },
  },
  en: {
    start: {
      title: 'Quick start',
      content: `\`neo-runner\` is a completion gate for coding agents. YAML describes the loop, JSON is the evidence, and only exit 0 may be called done.

The product is **binary + Skill + Hook**, not MCP.

## Install

\`\`\`bash
${INSTALL_CMD}
neo-runner --version
\`\`\`

Write the loop file inside the target repo:

\`\`\`bash
${INIT_CMD}
\`\`\`

Replace the placeholder commands in \`.agents/loop.yaml\` with the real gates. The hook skips if that file is missing.

Binary only:

\`\`\`bash
${SKIP_PLUGINS_CMD}
\`\`\`

No prebuilt package:

\`\`\`bash
${CARGO_CMD}
\`\`\`

Uninstall the global install (project \`.agents/loop.yaml\` is left alone):

\`\`\`bash
${UNINSTALL_CMD}
\`\`\`

## Verify

\`\`\`bash
${RUN_CMD}
\`\`\`

- Exit \`0\` and \`ok: true\`: done is allowed
- Exit \`1\`: red. JSON is still on stdout — read \`failed_tasks\` and \`evidence[].excerpt\`
- Exit \`2\`: missing config or YAML failed to load
`,
    },
    contract: {
      title: 'Agent contract',
      content: `\`neo-runner\` guarantees one thing: an agent cannot claim done without evidence.

## Command

\`\`\`bash
${RUN_CMD}
\`\`\`

- Exit \`0\`: \`ok\` is \`true\`
- Exit \`1\`: a task failed (JSON still goes to stdout)
- Exit \`2\`: missing or unloadable config

\`validate\` / \`plan\` only check YAML. They are not a completion signal.

## JSON fields

Agents should depend only on:

| Field | Type | Meaning |
| --- | --- | --- |
| \`ok\` | bool | The only green light |
| \`failed_tasks\` | string[] | Task ids that still failed |
| \`evidence\` | object[] | Failure evidence: \`task\`, \`exit_code\`, \`excerpt\` |
| \`duration_ms\` | number | Wall-clock time of the run |

\`excerpt\` prefers stderr, then stdout, then the error message. Max 2000 characters, tail kept.

## Loop file

Definition of Done lives in \`.agents/loop.yaml\`. Write it with \`neo-runner init\`.

## Hook

Claude Code Stop hook:

- No \`.agents/loop.yaml\`: skip (exit 0)
- Loop file exists but the binary does not: block (exit 1)
- Red run: block and print JSON to stdout
`,
    },
    harness: {
      title: 'Harness install',
      content: `One binary, three instruction sheets. Users do not need a local clone. Do not build a Claude-only MCP.

## One command

\`\`\`bash
${INSTALL_CMD}
\`\`\`

This:

- installs \`neo-runner\` from GitHub Releases into \`~/.local/bin\`
- tries to add that directory to \`PATH\`
- installs the marketplace plugin when \`claude\` CLI is present
- writes user-level skills

Binary only:

\`\`\`bash
${SKIP_PLUGINS_CMD}
\`\`\`

Uninstall the global install (project \`.agents/loop.yaml\` is left alone):

\`\`\`bash
${UNINSTALL_CMD}
\`\`\`

## Project loop file

Global install does not write the current directory. In the repo:

\`\`\`bash
${INIT_CMD}
\`\`\`

Existing \`.agents/loop.yaml\` is refused unless \`--force\`. Project-level skill:

\`\`\`bash
neo-runner init --skill
\`\`\`

## Claude Code

Without the \`claude\` CLI, type this in the chat:

\`\`\`text
/plugin marketplace add fzf54122/neo-runner
/plugin marketplace update neo-runner
/plugin install neo-runner@neo-runner
\`\`\`

The Stop hook loads only with the marketplace plugin.

Do not turn neo-runner into an MCP server unless the client has no Bash.
`,
    },
  },
}

export const yamlSample = `version: 1
job:
  name: agent-loop
  fail_fast: true
  tasks:
    - id: fmt
      type: shell
      cmd: cargo fmt --check
    - id: test
      type: shell
      depends_on: [fmt]
      cmd: cargo test`

export const jsonSample = `{
  "ok": true,
  "failed_tasks": [],
  "evidence": [],
  "duration_ms": 1420
}`

export const terminalLines = [
  { kind: 'cmd', text: `$ ${RUN_CMD}` },
  { kind: 'meta', text: 'neo-runner v0.2.0' },
  { kind: 'ok', text: '✓ Loading configuration' },
  { kind: 'ok', text: '✓ Resolving dependencies' },
  { kind: 'ok', text: '✓ Running tasks' },
  { kind: 'bar', text: '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ 100%' },
  { kind: 'done', text: 'Completed in 1.42s' },
]
