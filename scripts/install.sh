#!/usr/bin/env bash
# @Time    : 2026/9/17 16:20
# @Author  : fzf
# @FileName: install.sh
# @Software: neo-runner
#
# Global install: binary + Claude/Codex plugins. Does not write the current
# working directory. Project loops are created with `neo-runner init`.

set -euo pipefail

repo="fzf54122/neo-runner"
tag="${NEO_RUNNER_TAG:-latest}"
prefix="${NEO_RUNNER_PREFIX:-${HOME}/.local/bin}"
skip_plugins="${NEO_RUNNER_SKIP_PLUGINS:-0}"
skip_path="${NEO_RUNNER_SKIP_PATH:-0}"
force="${NEO_RUNNER_FORCE:-0}"
repo_dir="${NEO_RUNNER_REPO_DIR:-}"
download_url="${NEO_RUNNER_DOWNLOAD_URL:-}"

raw_ref="main"
if [ "${tag}" != "latest" ]; then
  raw_ref="${tag}"
fi
raw_base="${NEO_RUNNER_RAW_BASE:-https://raw.githubusercontent.com/${repo}/${raw_ref}}"

os="$(uname -s)"
arch="$(uname -m)"

case "${os}:${arch}" in
  Linux:x86_64|Linux:amd64)
    asset="neo-runner-linux-x86_64.tar.gz"
    binary="neo-runner-linux-x86_64"
    ;;
  MINGW*|MSYS*|CYGWIN*|Windows_NT*)
    echo "Windows: download neo-runner.exe from https://github.com/${repo}/releases" >&2
    exit 1
    ;;
  *)
    echo "unsupported platform ${os}/${arch}; use cargo install --git https://github.com/${repo} --bin neo-runner" >&2
    exit 1
    ;;
esac

if [ -z "${download_url}" ]; then
  if [ "${tag}" = "latest" ]; then
    download_url="https://github.com/${repo}/releases/latest/download/${asset}"
  else
    download_url="https://github.com/${repo}/releases/download/${tag}/${asset}"
  fi
fi

fetch() {
  src="$1"
  dest="$2"
  if [ -n "${repo_dir}" ]; then
    cp "${repo_dir}/${src}" "${dest}"
  else
    curl -fsSL "${raw_base}/${src}" -o "${dest}"
  fi
}

write_file() {
  dest="$1"
  src_rel="$2"
  if [ -f "${dest}" ] && [ "${force}" != "1" ]; then
    echo "keep ${dest}"
    return 0
  fi
  mkdir -p "$(dirname "${dest}")"
  fetch "${src_rel}" "${dest}"
  echo "wrote ${dest}"
}

path_contains_prefix() {
  case ":${PATH}:" in
    *":${prefix}:"*) return 0 ;;
    *) return 1 ;;
  esac
}

ensure_path() {
  already_on_path=0
  if path_contains_prefix; then
    already_on_path=1
  fi
  export PATH="${prefix}:${PATH}"
  if [ "${skip_path}" = "1" ] || [ "${already_on_path}" = "1" ]; then
    return 0
  fi

  marker="# neo-runner"
  line="export PATH=\"${prefix}:\$PATH\" ${marker}"
  for rc in "${HOME}/.zprofile" "${HOME}/.zshrc" "${HOME}/.bashrc" "${HOME}/.profile"; do
    if [ -f "${rc}" ] && [ -w "${rc}" ]; then
      if grep -F "${marker}" "${rc}" >/dev/null 2>&1; then
        echo "PATH already configured in ${rc}"
        return 0
      fi
      printf '\n%s\n' "${line}" >> "${rc}"
      echo "Added ${prefix} to PATH in ${rc}"
      return 0
    fi
  done

  for rc in "${HOME}/.zprofile" "${HOME}/.profile"; do
    if [ ! -e "${rc}" ]; then
      printf '%s\n' "${line}" > "${rc}"
      echo "Created ${rc} and added ${prefix} to PATH"
      return 0
    fi
  done

  echo "Make sure ${prefix} is on PATH." >&2
}

install_binary() {
  tmpdir="$(mktemp -d)"
  trap 'rm -rf "${tmpdir}"' EXIT

  echo "Downloading ${download_url}"
  if [ -f "${download_url}" ]; then
    cp "${download_url}" "${tmpdir}/${asset}"
  else
    curl -fsSL "${download_url}" -o "${tmpdir}/${asset}"
  fi
  tar -xzf "${tmpdir}/${asset}" -C "${tmpdir}"

  mkdir -p "${prefix}"
  install -m 0755 "${tmpdir}/${binary}" "${prefix}/neo-runner"
  echo "Installed ${prefix}/neo-runner"
  "${prefix}/neo-runner" --version
}

install_claude_plugin() {
  if ! command -v claude >/dev/null 2>&1; then
    echo "claude CLI not found; wrote user skill only."
    echo "In Claude Code run:"
    echo "  /plugin marketplace add ${repo}"
    echo "  /plugin install fzf54122@neo-runner"
    return 0
  fi

  if claude plugin marketplace add "${repo}"; then
    :
  else
    echo "claude plugin marketplace add failed (already added is ok)." >&2
  fi

  if claude plugin install fzf54122@neo-runner --scope user -y; then
    echo "Installed Claude plugin fzf54122@neo-runner (user scope)."
  else
    echo "claude plugin install failed. In Claude Code run:" >&2
    echo "  /plugin marketplace add ${repo}" >&2
    echo "  /plugin install fzf54122@neo-runner" >&2
  fi
}

install_skills() {
  write_file "${HOME}/.claude/skills/neo-runner/SKILL.md" "skills/neo-runner/SKILL.md"

  if command -v codex >/dev/null 2>&1 || [ -d "${HOME}/.codex" ]; then
    write_file "${HOME}/.codex/skills/neo-runner/SKILL.md" ".agents/skills/neo-runner/SKILL.md"
  fi

  write_file "${HOME}/.agents/skills/neo-runner/SKILL.md" ".agents/skills/neo-runner/SKILL.md"
}

install_binary
ensure_path

if [ "${skip_plugins}" != "1" ]; then
  install_skills
  install_claude_plugin
fi

echo
echo "Next: in a project, run: neo-runner init"
echo "Then replace echo placeholders in .agents/loop.yaml."
