#!/usr/bin/env bash
# Build dependencies for OpenBlade-Synapse (Fedora). Other distros: see README ("Build from source").
set -euo pipefail

echo "==> System packages: C++ compiler and Qt 6 development files"
sudo dnf install -y gcc-c++ qt6-qtbase-devel qt6-qtdeclarative-devel

if ! command -v cargo >/dev/null 2>&1 && [ ! -x "$HOME/.cargo/bin/cargo" ]; then
  echo "==> Rust (rustup, installs into ~/.cargo – no sudo)"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
fi

# shellcheck disable=SC1091
source "$HOME/.cargo/env"
echo
rustc --version
cargo --version
qmake6 --version | head -1
g++ --version | head -1
echo "==> Done. Now run ./build.sh"
