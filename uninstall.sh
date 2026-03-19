#!/bin/bash
# claude-sync uninstaller for Linux
# Usage: chmod +x uninstall.sh && ./uninstall.sh

set -e

echo "Uninstalling claude-sync..."

rm -f ~/.local/bin/claude-sync.AppImage
rm -f ~/.local/share/applications/claude-sync.desktop
rm -f ~/.local/share/icons/claude-sync.png
update-desktop-database ~/.local/share/applications/ 2>/dev/null || true

echo ""
read -p "Also delete sync data (~/.claude-sync)? [y/N] " choice
if [[ "$choice" =~ ^[Yy]$ ]]; then
    rm -rf ~/.claude-sync
    echo "Sync data removed."
fi

echo "Done. claude-sync has been uninstalled."
