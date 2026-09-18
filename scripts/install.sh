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

os="${NEO_RUNNER_OS:-$(uname -s)}"
arch="${NEO_RUNNER_ARCH:-$(uname -m)}"
os_lc="$(printf '%s' "${os}" | tr '[:upper:]' '[:lower:]')"
arch_lc="$(printf '%s' "${arch}" | tr '[:upper:]' '[:lower:]')"
extract=""
installed_name=""
asset=""
binary=""

case "${os_lc}" in
  linux)
    case "${arch_lc}" in
      x86_64|amd64)
        asset="neo-runner-linux-x86_64.tar.gz"
        binary="neo-runner-linux-x86_64"
        extract="tar"
        installed_name="neo-runner"
        ;;
    esac
    ;;
  mingw*|msys*|cygwin*|windows_nt*|windows)
    case "${arch_lc}" in
      x86_64|amd64)
        asset="neo-runner.exe"
        binary="neo-runner.exe"
        extract="file"
        installed_name="neo-runner.exe"
        ;;
    esac
    ;;
esac

if [ "${NEO_RUNNER_DETECT_ONLY:-0}" = "1" ]; then
  echo "os=${os}"
  echo "arch=${arch}"
  echo "asset=${asset}"
  echo "binary=${binary}"
  echo "installed=${installed_name}"
  if [ -n "${asset}" ]; then
    exit 0
  fi
  echo "unsupported platform ${os}/${arch}" >&2
  exit 1
fi

if [ -z "${asset}" ]; then
  echo "unsupported platform ${os}/${arch}; use cargo install --git https://github.com/${repo} --bin neo-runner" >&2
  exit 1
fi

release_api_url() {
  if [ "${tag}" = "latest" ]; then
    printf 'https://api.github.com/repos/%s/releases/latest' "${repo}"
  else
    printf 'https://api.github.com/repos/%s/releases/tags/%s' "${repo}" "${tag}"
  fi
}

constructed_download_url() {
  if [ "${tag}" = "latest" ]; then
    printf 'https://github.com/%s/releases/latest/download/%s' "${repo}" "${asset}"
  else
    printf 'https://github.com/%s/releases/download/%s/%s' "${repo}" "${tag}" "${asset}"
  fi
}

resolve_download_url() {
  if [ -n "${download_url}" ]; then
    return 0
  fi

  api="$(release_api_url)"
  json="$(curl -fsSL "${api}" || true)"
  if [ -n "${json}" ] && command -v python3 >/dev/null 2>&1; then
    resolved="$(printf '%s' "${json}" | python3 -c '
import json, sys
want = sys.argv[1]
rel = json.load(sys.stdin)
print(rel.get("tag_name", ""))
url = ""
for asset in rel.get("assets") or []:
    if asset.get("name") == want:
        url = asset.get("browser_download_url") or ""
        break
print(url)
' "${asset}" || true)"
    resolved_tag="$(printf '%s\n' "${resolved}" | sed -n '1p')"
    resolved_url="$(printf '%s\n' "${resolved}" | sed -n '2p')"
    if [ -n "${resolved_tag}" ]; then
      echo "Latest release ${resolved_tag} for ${os}/${arch}"
    fi
    if [ -n "${resolved_url}" ]; then
      download_url="${resolved_url}"
      return 0
    fi
    echo "Release ${resolved_tag:-${tag}} has no asset ${asset} for ${os}/${arch}" >&2
    echo "Use: cargo install --git https://github.com/${repo} --bin neo-runner" >&2
    exit 1
  fi

  download_url="$(constructed_download_url)"
}

resolve_download_url

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

  src="${tmpdir}/${binary}"
  case "${extract}" in
    tar)
      tar -xzf "${tmpdir}/${asset}" -C "${tmpdir}"
      ;;
    zip)
      if command -v unzip >/dev/null 2>&1; then
        unzip -qo "${tmpdir}/${asset}" -d "${tmpdir}"
      else
        python3 - "${tmpdir}/${asset}" "${tmpdir}" <<'PY'
import sys
import zipfile

zipfile.ZipFile(sys.argv[1]).extractall(sys.argv[2])
PY
      fi
      ;;
    file)
      src="${tmpdir}/${asset}"
      ;;
    *)
      echo "unknown archive type ${extract}" >&2
      exit 1
      ;;
  esac

  if [ ! -f "${src}" ]; then
    echo "downloaded ${asset}, but ${binary} was not in the archive" >&2
    exit 1
  fi

  mkdir -p "${prefix}"
  dest="${prefix}/${installed_name}"
  cp "${src}" "${dest}"
  chmod 0755 "${dest}" 2>/dev/null || true
  echo "Installed ${dest}"
  "${dest}" --version
}

install_claude_plugin() {
  # plugin@marketplace：右边是 marketplace.json 的 name，不是 GitHub 用户名。
  # 本机第一次 add 时登记的名字不会随远程改名而变；老用户是 neo-runner。
  marketplace_name="neo-runner"
  plugin_id="neo-runner@neo-runner"
  # 短窗口里 marketplace.json 曾改成 fzf54122，那批本机登记名不同。
  legacy_marketplace_name="fzf54122"
  legacy_plugin_id="neo-runner@fzf54122"

  if ! command -v claude >/dev/null 2>&1; then
    echo "claude CLI not found; wrote user skill only."
    echo "In Claude Code run:"
    echo "  /plugin marketplace add ${repo}"
    echo "  /plugin marketplace update ${marketplace_name}"
    echo "  /plugin install ${plugin_id}"
    return 0
  fi

  if claude plugin marketplace add "${repo}"; then
    :
  else
    echo "claude plugin marketplace add failed (already added is ok)." >&2
  fi

  # add 在本机已有副本时是 no-op，必须再 update 才能拿到最新 marketplace.json。
  # 已登记的 marketplace 名不会随远程改 name 而变，所以两种名字都试。
  for name in "${marketplace_name}" "${legacy_marketplace_name}"; do
    if claude plugin marketplace update "${name}"; then
      :
    else
      echo "claude plugin marketplace update ${name} failed (continuing)." >&2
    fi
  done

  # version 未变时 `plugin update` 会跳过，旧 cache 里重复声明的 hooks 清不掉。
  for id in "${plugin_id}" "${legacy_plugin_id}"; do
    claude plugin uninstall "${id}" --scope user -y >/dev/null 2>&1 || true
  done

  if claude plugin install "${plugin_id}" --scope user -y; then
    echo "Installed Claude plugin ${plugin_id} (user scope)."
  elif claude plugin install "${legacy_plugin_id}" --scope user -y; then
    echo "Installed Claude plugin ${legacy_plugin_id} (user scope, legacy marketplace name)."
  else
    echo "claude plugin install failed. In Claude Code run:" >&2
    echo "  /plugin marketplace add ${repo}" >&2
    echo "  /plugin marketplace update ${marketplace_name}" >&2
    echo "  /plugin install ${plugin_id}" >&2
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
