#!/usr/bin/env sh
# @Time    : 2026/9/17 10:52
# @Author  : fzf
# @FileName: stop-gate.sh
# @Software: neo-runner
#
# Claude Code Stop hook. Blocks session end when a project loop is red.
# No loop file → skip (exit 0). Loop exists but binary missing → fail (exit 1).

set -eu

loop_file="${NEO_RUNNER_LOOP:-.agents/loop.yaml}"

if [ ! -f "$loop_file" ]; then
  echo "neo-runner: skip gate (no $loop_file)"
  exit 0
fi

if command -v neo-runner >/dev/null 2>&1; then
  runner="neo-runner"
elif [ -x "./target/release/neo-runner" ]; then
  runner="./target/release/neo-runner"
elif [ -x "./target/debug/neo-runner" ]; then
  runner="./target/debug/neo-runner"
else
  echo "neo-runner: binary not found. Install with: curl -fsSL https://raw.githubusercontent.com/fzf54122/neo-runner/main/scripts/install.sh | bash" >&2
  echo "Loop file exists ($loop_file) so completion is blocked until the binary is available." >&2
  exit 1
fi

output="$("$runner" run -f "$loop_file" --output json)" || status=$?
status="${status:-0}"
printf '%s\n' "$output"

if [ "$status" -ne 0 ]; then
  echo "neo-runner: loop is red. Do not claim the work is done. Fix failed_tasks and rerun." >&2
  exit 1
fi

exit 0
