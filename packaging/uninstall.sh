#!/usr/bin/env bash
# Remove OpenBlade-Synapse from the application menu and autostart (settings in ~/.config are kept).
APPS="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
rm -f "$APPS/openblade-synapse.desktop" "${XDG_CONFIG_HOME:-$HOME/.config}/autostart/openblade-synapse.desktop"
update-desktop-database "$APPS" 2>/dev/null || true
echo "Removed. Delete this folder to remove the program; settings: ~/.config/openblade-synapse"
