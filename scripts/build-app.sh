#!/usr/bin/env bash
# Compila PaintCP per il Mac corrente e crea il bundle "PaintCP.app" in target/.
set -euo pipefail

cd "$(dirname "$0")/.."

cargo build --release
PAINTCP_VERSION="${PAINTCP_VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)}" \
    ./scripts/bundle-app.sh target/release/paintcp target

echo "Avvio: open target/PaintCP.app"
