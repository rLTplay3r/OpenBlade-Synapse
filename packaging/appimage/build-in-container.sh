#!/usr/bin/env bash
# Runs INSIDE the build container (see make-appimage.sh): compile, assemble the AppDir, pack the AppImage.
# /src = the project (read-write), /cache = cargo cache + target dir, /out = where the AppImage goes.
set -euo pipefail
export QMAKE=/opt/qt/current/bin/qmake
export CARGO_TARGET_DIR=/cache/target
export CARGO_HOME=/cache/cargo   # downloaded crates are kept between runs (the toolchain stays in /opt)

VERSION=$(sed -n 's/^pub const VERSION: &str = "\(.*\)";/\1/p' /src/rust/obcore/src/lib.rs)
echo "==> [1/3] Compiling OpenBlade-Synapse $VERSION (Qt $($QMAKE -query QT_VERSION))"
cd /src/rust
cargo build --release --workspace --locked

echo "==> [2/3] Assembling the AppDir"
APPDIR=/tmp/AppDir
rm -rf "$APPDIR"
mkdir -p "$APPDIR/usr/bin" "$APPDIR/usr/app"
cp "$CARGO_TARGET_DIR/release/OpenBlade-Synapse" "$CARGO_TARGET_DIR/release/openblade-synapse-gui" "$APPDIR/usr/bin/"
for d in qml qml-rust assets i18n devices; do cp -r "/src/app/$d" "$APPDIR/usr/app/"; done
cp /src/LICENSE "$APPDIR/usr/app/"
cat > /tmp/openblade-synapse.desktop <<DESKTOP
[Desktop Entry]
Type=Application
Name=OpenBlade-Synapse
Comment=Razer mouse settings (OpenRazer)
Exec=OpenBlade-Synapse
Icon=openblade-synapse
Terminal=false
Categories=Settings;HardwareSettings;
StartupWMClass=openblade-synapse
DESKTOP
cp /src/app/assets/icon.png /tmp/openblade-synapse.png

PLUGINS=/opt/qt/current/plugins
EXTRA=""
for p in "$PLUGINS"/platforms/libqwayland*.so; do [ -e "$p" ] && EXTRA="$EXTRA;$(basename "$p")"; done
export EXTRA_PLATFORM_PLUGINS="${EXTRA#;}"
export QML_SOURCES_PATHS="/src/app/qml:/src/app/qml-rust"
export LD_LIBRARY_PATH=/opt/qt/current/lib
echo "    extra platform plugins: ${EXTRA_PLATFORM_PLUGINS:-none}"
linuxdeploy-x86_64.AppImage --appdir "$APPDIR" \
  -e "$APPDIR/usr/bin/OpenBlade-Synapse" -e "$APPDIR/usr/bin/openblade-synapse-gui" \
  -d /tmp/openblade-synapse.desktop -i /tmp/openblade-synapse.png --plugin qt

# Wayland: linuxdeploy-plugin-qt deploys the platform plugin but not its sub-plugins; without the EGL client
# integration Qt Quick cannot create an OpenGL context on Wayland. Their rpath ($ORIGIN/../../lib) fits the AppDir.
for d in wayland-graphics-integration-client wayland-shell-integration wayland-decoration-client; do
  [ -d "$PLUGINS/$d" ] && mkdir -p "$APPDIR/usr/plugins/$d" && cp -n "$PLUGINS/$d"/*.so "$APPDIR/usr/plugins/$d/"
done

echo "==> [3/3] Packing the AppImage"
OUT="/out/OpenBlade-Synapse-v$VERSION-x86_64.AppImage"
ARCH=x86_64 appimagetool-x86_64.AppImage --no-appstream --runtime-file /opt/tools/runtime-x86_64 "$APPDIR" "$OUT.tmp"
mv -f "$OUT.tmp" "$OUT"
echo "==> $OUT ($(du -h "$OUT" | cut -f1))"
