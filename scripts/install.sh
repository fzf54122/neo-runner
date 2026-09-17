#!/usr/bin/env bash
# @Time    : 2026/9/17 13:30
# @Author  : fzf
# @FileName: install.sh
# @Software: neo-runner
#
# Install neo-runner from GitHub Releases. No local clone required.

set -euo pipefail

repo="fzf54122/neo-runner"
tag="${NEO_RUNNER_TAG:-latest}"
prefix="${NEO_RUNNER_PREFIX:-${HOME}/.local/bin}"

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

if [ "${tag}" = "latest" ]; then
  url="https://github.com/${repo}/releases/latest/download/${asset}"
else
  url="https://github.com/${repo}/releases/download/${tag}/${asset}"
fi

tmpdir="$(mktemp -d)"
trap 'rm -rf "${tmpdir}"' EXIT

echo "Downloading ${url}"
curl -fsSL "${url}" -o "${tmpdir}/${asset}"
tar -xzf "${tmpdir}/${asset}" -C "${tmpdir}"

mkdir -p "${prefix}"
install -m 0755 "${tmpdir}/${binary}" "${prefix}/neo-runner"

echo "Installed ${prefix}/neo-runner"
"${prefix}/neo-runner" --version
echo "Make sure ${prefix} is on PATH."
