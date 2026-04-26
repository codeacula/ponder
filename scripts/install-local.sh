#!/usr/bin/env sh
set -eu

cd "$(dirname "$0")/.."

cargo install --path . --force

echo "Installed ponder to $(command -v ponder)"
