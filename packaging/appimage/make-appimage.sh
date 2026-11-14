#!/usr/bin/env bash
# Build dist/OpenBlade-Synapse-v<version>-x86_64.AppImage in a container (podman or docker).
# The first run downloads the build environment (Ubuntu 22.04 + Qt + Rust, a few GB) – later runs reuse it.
set -euo pipefail
ROOT="$(cd "$(dirname "$(readlink -f "$0")")/../.." && pwd)"
ENGINE="${CONTAINER_ENGINE:-}"
if [ -z "$ENGINE" ]; then
  if command -v podman >/dev/null 2>&1; then ENGINE=podman
  elif command -v docker >/dev/null 2>&1; then ENGINE=docker
  else echo "!! podman or docker is needed to build the AppImage"; exit 1; fi
fi
IMAGE=openblade-appimage-builder:1
mkdir -p "$ROOT/dist"
echo "==> Build environment ($ENGINE image $IMAGE)"
if ! "$ENGINE" image exists "$IMAGE" 2>/dev/null && [ -z "$("$ENGINE" images -q "$IMAGE" 2>/dev/null)" ]; then
  echo "    First AppImage build: downloading Ubuntu 22.04 + Qt + Rust (a few GB, 10-20 minutes). Later builds reuse it."
fi
"$ENGINE" build -t "$IMAGE" -f "$ROOT/packaging/appimage/Containerfile" "$ROOT/packaging/appimage"
"$ENGINE" run --rm --security-opt label=disable \
  -v "$ROOT":/src -v "$ROOT/dist":/out -v openblade-appimage-cache:/cache \
  "$IMAGE" bash /src/packaging/appimage/build-in-container.sh
