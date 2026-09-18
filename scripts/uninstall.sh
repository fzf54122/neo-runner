#!/usr/bin/env bash
# @Time    : 2026/9/18 11:51
# @Author  : fzf
# @FileName: uninstall.sh
# @Software: neo-runner
#
# Reverse of scripts/install.sh. Removes the global binary, PATH marker,
# user-level skills, and Claude plugin / marketplace leftovers.
# Does not touch the current working directory (project loops from
# `neo-runner init` stay). Idempotent.

set -euo pipefail

repo="fzf54122/neo-runner"
prefix="${NEO_RUNNER_PREFIX:-${HOME}/.local/bin}"
skip_plugins="${NEO_RUNNER_SKIP_PLUGINS:-0}"
skip_path="${NEO_RUNNER_SKIP_PATH:-0}"
skip_claude="${NEO_RUNNER_SKIP_CLAUDE:-0}"
skip_deb="${NEO_RUNNER_SKIP_DEB:-0}"
dry_run="${NEO_RUNNER_DRY_RUN:-0}"
purge_deb="${NEO_RUNNER_PURGE_DEB:-0}"
cargo_home="${CARGO_HOME:-${HOME}/.cargo}"
marker="# neo-runner"

if [ -z "${HOME:-}" ] || [ "${HOME}" = "/" ]; then
  echo "refusing to uninstall with HOME='${HOME:-}'" >&2
  exit 1
fi
if [ -z "${prefix}" ] || [ "${prefix}" = "/" ]; then
  echo "refusing to uninstall with prefix='${prefix}'" >&2
  exit 1
fi
if [ -z "${cargo_home}" ] || [ "${cargo_home}" = "/" ]; then
  echo "refusing to uninstall with CARGO_HOME='${cargo_home}'" >&2
  exit 1
fi

usage() {
  cat <<EOF
Usage: uninstall.sh [--dry-run] [--purge-deb] [--keep-path] [--keep-plugins]

Removes the global neo-runner install created by scripts/install.sh.
Does not delete project files such as .agents/loop.yaml.

Options:
  --dry-run        Print actions only
  --purge-deb      Also run sudo dpkg -P neo-runner
  --keep-path      Leave shell PATH lines in place
  --keep-plugins   Leave Claude plugin, marketplace, and user skills
  -h, --help       Show this help

Environment:
  NEO_RUNNER_PREFIX        Binary dir (default: ~/.local/bin)
  NEO_RUNNER_SKIP_PLUGINS  1 = same as --keep-plugins
  NEO_RUNNER_SKIP_PATH     1 = same as --keep-path
  NEO_RUNNER_SKIP_CLAUDE   1 = do not invoke the claude CLI
  NEO_RUNNER_SKIP_DEB      1 = do not inspect or purge the Debian package
  NEO_RUNNER_DRY_RUN       1 = same as --dry-run
  NEO_RUNNER_PURGE_DEB     1 = same as --purge-deb
  CARGO_HOME               Cargo bin dir parent (default: ~/.cargo)
EOF
}

while [ $# -gt 0 ]; do
  case "$1" in
    --dry-run) dry_run=1 ;;
    --purge-deb) purge_deb=1 ;;
    --keep-path) skip_path=1 ;;
    --keep-plugins) skip_plugins=1 ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
  shift
done

removed=0
skipped=0
warnings=0

say() {
  printf '%s\n' "$*"
}

would() {
  if [ "${dry_run}" = "1" ]; then
    printf 'would %s\n' "$*"
  else
    printf '%s\n' "$*"
  fi
}

remove_file() {
  path="$1"
  if [ ! -e "${path}" ] && [ ! -L "${path}" ]; then
    return 0
  fi
  if [ "${dry_run}" = "1" ]; then
    would "remove ${path}"
    removed=$((removed + 1))
    return 0
  fi
  if [ -e "${path}" ] && [ ! -w "${path}" ] && [ ! -w "$(dirname "${path}")" ]; then
    say "skip ${path} (not writable)" >&2
    skipped=$((skipped + 1))
    return 0
  fi
  if ! rm -f "${path}"; then
    say "skip ${path} (could not remove)" >&2
    skipped=$((skipped + 1))
    return 0
  fi
  would "removed ${path}"
  removed=$((removed + 1))
}

remove_dir() {
  path="$1"
  if [ ! -e "${path}" ]; then
    return 0
  fi
  if [ "${dry_run}" = "1" ]; then
    would "remove ${path}"
    removed=$((removed + 1))
    return 0
  fi
  if ! rm -rf "${path}"; then
    say "skip ${path} (could not remove)" >&2
    skipped=$((skipped + 1))
    return 0
  fi
  would "removed ${path}"
  removed=$((removed + 1))
}

strip_path_marker() {
  rc="$1"
  if [ ! -f "${rc}" ]; then
    return 0
  fi
  if ! grep -F "${marker}" "${rc}" >/dev/null 2>&1; then
    return 0
  fi
  if [ "${dry_run}" = "1" ]; then
    would "strip PATH marker from ${rc}"
    removed=$((removed + 1))
    return 0
  fi
  if [ ! -w "${rc}" ]; then
    say "skip ${rc} (not writable)" >&2
    skipped=$((skipped + 1))
    return 0
  fi
  tmp="$(mktemp)"
  grep -Fv "${marker}" "${rc}" > "${tmp}" || true
  mv "${tmp}" "${rc}"
  would "stripped PATH marker from ${rc}"
  removed=$((removed + 1))
}

run_claude() {
  if [ "${skip_claude}" = "1" ]; then
    return 1
  fi
  if ! command -v claude >/dev/null 2>&1; then
    return 1
  fi
  claude "$@"
}

prune_plugin_registry() {
  file="$1"
  kind="$2"
  if [ ! -f "${file}" ]; then
    return 0
  fi
  if ! command -v python3 >/dev/null 2>&1; then
    return 0
  fi
  if [ "${dry_run}" = "1" ]; then
    would "prune ${kind} entries from ${file}"
    removed=$((removed + 1))
    return 0
  fi
  if python3 - "${file}" "${kind}" <<'PY'
import json
import sys

path, kind = sys.argv[1], sys.argv[2]
try:
    with open(path, encoding="utf-8") as fh:
        data = json.load(fh)
except (OSError, json.JSONDecodeError):
    sys.exit(2)

changed = False
if kind == "plugins":
    plugins = data.get("plugins") if isinstance(data, dict) else None
    if isinstance(plugins, dict):
        drop = [key for key in plugins if "neo-runner" in key]
        for key in drop:
            del plugins[key]
            changed = True
elif kind == "marketplaces":
    if isinstance(data, dict):
        for key in ("neo-runner", "fzf54122"):
            if key in data:
                del data[key]
                changed = True

if changed:
    with open(path, "w", encoding="utf-8") as fh:
        json.dump(data, fh, indent=2)
        fh.write("\n")
    sys.exit(0)
sys.exit(2)
PY
  then
    would "pruned ${kind} entries from ${file}"
    removed=$((removed + 1))
  fi
}

uninstall_binary() {
  remove_file "${prefix}/neo-runner"
  remove_file "${cargo_home}/bin/neo-runner"

  for completion in \
    "${HOME}/.local/share/bash-completion/completions/neo-runner" \
    "${HOME}/.local/share/zsh/site-functions/_neo-runner" \
    "${HOME}/.config/fish/completions/neo-runner.fish"
  do
    remove_file "${completion}"
  done
}

uninstall_path() {
  if [ "${skip_path}" = "1" ]; then
    say "keep PATH lines"
    return 0
  fi
  for rc in "${HOME}/.zprofile" "${HOME}/.zshrc" "${HOME}/.bashrc" "${HOME}/.profile"; do
    strip_path_marker "${rc}"
  done
}

uninstall_skills() {
  remove_dir "${HOME}/.claude/skills/neo-runner"
  remove_dir "${HOME}/.codex/skills/neo-runner"
  remove_dir "${HOME}/.agents/skills/neo-runner"
}

uninstall_claude_plugin() {
  if [ "${skip_claude}" = "1" ]; then
    say "skip claude CLI"
  elif command -v claude >/dev/null 2>&1; then
    for plugin_id in "neo-runner@fzf54122" "neo-runner@neo-runner"; do
      if [ "${dry_run}" = "1" ]; then
        would "claude plugin uninstall ${plugin_id} --scope user"
        continue
      fi
      if run_claude plugin uninstall "${plugin_id}" --scope user -y >/dev/null 2>&1; then
        would "uninstalled Claude plugin ${plugin_id} (user scope)"
        removed=$((removed + 1))
      fi
    done

    for marketplace in "neo-runner" "fzf54122"; do
      if [ "${dry_run}" = "1" ]; then
        would "claude plugin marketplace remove ${marketplace}"
        continue
      fi
      if run_claude plugin marketplace remove "${marketplace}" >/dev/null 2>&1; then
        would "removed Claude marketplace ${marketplace}"
        removed=$((removed + 1))
      fi
    done
  else
    say "claude CLI not found; removing plugin files only."
  fi

  remove_dir "${HOME}/.claude/plugins/cache/neo-runner"
  remove_dir "${HOME}/.claude/plugins/data/neo-runner-neo-runner"
  remove_dir "${HOME}/.claude/plugins/marketplaces/neo-runner"
  prune_plugin_registry "${HOME}/.claude/plugins/installed_plugins.json" "plugins"
  prune_plugin_registry "${HOME}/.claude/plugins/known_marketplaces.json" "marketplaces"
}

maybe_purge_deb() {
  if [ "${skip_deb}" = "1" ]; then
    return 0
  fi
  if ! command -v dpkg >/dev/null 2>&1; then
    return 0
  fi
  if ! dpkg-query -W -f='${Status}' neo-runner 2>/dev/null | grep -q "installed"; then
    return 0
  fi
  if [ "${purge_deb}" != "1" ]; then
    say "Debian package neo-runner is installed. Purge with:"
    say "  sudo dpkg -P neo-runner"
    say "or rerun: $0 --purge-deb"
    warnings=$((warnings + 1))
    return 0
  fi
  if [ "${dry_run}" = "1" ]; then
    would "sudo dpkg -P neo-runner"
    removed=$((removed + 1))
    return 0
  fi
  if sudo dpkg -P neo-runner; then
    would "purged Debian package neo-runner"
    removed=$((removed + 1))
  else
    say "failed to purge Debian package neo-runner" >&2
    skipped=$((skipped + 1))
  fi
}

warn_leftovers() {
  leftovers=()
  for path in \
    "${prefix}/neo-runner" \
    "${cargo_home}/bin/neo-runner" \
    "${HOME}/.claude/skills/neo-runner" \
    "${HOME}/.codex/skills/neo-runner" \
    "${HOME}/.agents/skills/neo-runner" \
    "${HOME}/.claude/plugins/cache/neo-runner" \
    "${HOME}/.claude/plugins/data/neo-runner-neo-runner" \
    "${HOME}/.claude/plugins/marketplaces/neo-runner"
  do
    if [ -e "${path}" ]; then
      leftovers+=("${path}")
    fi
  done

  if command -v neo-runner >/dev/null 2>&1; then
    leftover_bin="$(command -v neo-runner)"
    case "${leftover_bin}" in
      "${prefix}/neo-runner"|"${cargo_home}/bin/neo-runner")
        ;;
      *)
        leftovers+=("${leftover_bin}")
        ;;
    esac
  fi

  if [ "${#leftovers[@]}" -gt 0 ]; then
    say
    say "Still present:"
    for path in "${leftovers[@]}"; do
      say "  ${path}"
    done
    warnings=$((warnings + 1))
  fi
}

uninstall_binary
uninstall_path

if [ "${skip_plugins}" != "1" ]; then
  uninstall_skills
  uninstall_claude_plugin
else
  say "keep plugins and user skills"
fi

maybe_purge_deb

if [ "${dry_run}" != "1" ]; then
  warn_leftovers
fi

say
say "Project files such as .agents/loop.yaml were not touched."
if [ "${dry_run}" = "1" ]; then
  say "Dry run complete (${removed} planned actions)."
else
  say "Uninstall complete (${removed} removed, ${skipped} skipped, ${warnings} warnings)."
fi
say "Reinstall: curl -fsSL https://raw.githubusercontent.com/${repo}/main/scripts/install.sh | bash"
