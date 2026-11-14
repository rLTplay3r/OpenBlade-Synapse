#!/usr/bin/env bash
# Build OpenBlade-Synapse from source into dist/OpenBlade-Synapse/ (a self-contained folder).
# Needs: Rust (cargo), a C++ compiler and the Qt 6 development packages (qmake6) – see README.
# Usage: ./build.sh            build
#        ./build.sh --install  build and add OpenBlade-Synapse to the application menu
set -euo pipefail
cd "$(dirname "$(readlink -f "$0")")"

command -v cargo >/dev/null 2>&1 || source "$HOME/.cargo/env" 2>/dev/null || { echo "cargo not found – install Rust (https://rustup.rs)"; exit 1; }

echo "==> Compiling (release)"
# GCC 16 warns inside Qt's own headers; harmless, silenced
(cd rust && CXXFLAGS="-Wno-sfinae-incomplete ${CXXFLAGS:-}" cargo build --release)

OUT=dist/OpenBlade-Synapse
echo "==> Assembling $OUT"
rm -rf "$OUT.new" && mkdir -p "$OUT.new"
cp rust/target/release/OpenBlade-Synapse rust/target/release/openblade-synapse-gui "$OUT.new/"
cp -r app/qml app/qml-rust app/assets app/i18n app/devices "$OUT.new/"
cp packaging/install.sh packaging/uninstall.sh LICENSE "$OUT.new/"
rm -rf "$OUT" && mv "$OUT.new" "$OUT"

if [ "${1:-}" = "--install" ]; then
  "$OUT/install.sh"
fi
echo "==> Done: $OUT/OpenBlade-Synapse"
