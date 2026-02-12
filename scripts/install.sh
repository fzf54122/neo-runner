#!/usr/bin/env bash
set -euo pipefail

echo "Installing neo-runner..."
cargo install --path crates/runner-cli
