#!/usr/bin/env bash
set -euo pipefail

echo "Preparing dev environment for neo-runner"
rustup component add rustfmt clippy
cargo fetch
