#!/usr/bin/env bash
# Add OpenBlade-Synapse to the application menu (no root needed).
# Run it from the extracted OpenBlade-Synapse folder; the program stays where it is.
set -euo pipefail
DIR="$(cd "$(dirname "$(readlink -f "$0")")" && pwd)"
APPS="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
mkdir -p "$APPS"
cat > "$APPS/openblade-synapse.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=OpenBlade-Synapse
Comment=Razer mouse settings (OpenRazer)
Exec="$DIR/OpenBlade-Synapse"
Icon=$DIR/assets/icon.png
Terminal=false
Categories=Settings;HardwareSettings;
StartupWMClass=openblade-synapse
DESKTOP
update-desktop-database "$APPS" 2>/dev/null || true
echo "OpenBlade-Synapse added to the application menu."

# friendly checks
if ! gdbus introspect --session --dest org.razer --object-path /org/razer >/dev/null 2>&1; then
  echo "! The OpenRazer daemon is not running – install OpenRazer: https://openrazer.github.io/#download"
fi
if ! id -nG | grep -qw plugdev; then
  echo "! You are not in the 'plugdev' group: sudo gpasswd -a \$USER plugdev  (then log out and in)"
fi
