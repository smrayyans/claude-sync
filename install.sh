#!/bin/bash
# claude-sync installer for Linux
# Usage: chmod +x install.sh && ./install.sh

set -e

APPIMAGE=$(ls claude-sync_*.AppImage 2>/dev/null | head -1)

if [ -z "$APPIMAGE" ]; then
    echo "Error: No claude-sync_*.AppImage found in current directory."
    echo "Download it first from: https://github.com/smrayyans/claude-sync/releases"
    exit 1
fi

echo "Installing claude-sync..."

mkdir -p ~/.local/bin ~/.local/share/applications ~/.local/share/icons

chmod +x "$APPIMAGE"
cp "$APPIMAGE" ~/.local/bin/claude-sync.AppImage

# Extract icon
cd /tmp
~/.local/bin/claude-sync.AppImage --appimage-extract usr/share/icons >/dev/null 2>&1 || true
if [ -f squashfs-root/usr/share/icons/hicolor/256x256@2/apps/claude-sync.png ]; then
    cp squashfs-root/usr/share/icons/hicolor/256x256@2/apps/claude-sync.png ~/.local/share/icons/claude-sync.png
elif [ -f squashfs-root/usr/share/icons/hicolor/128x128/apps/claude-sync.png ]; then
    cp squashfs-root/usr/share/icons/hicolor/128x128/apps/claude-sync.png ~/.local/share/icons/claude-sync.png
fi
rm -rf squashfs-root
cd - >/dev/null

# Create desktop entry
cat > ~/.local/share/applications/claude-sync.desktop << ENTRY
[Desktop Entry]
Name=Claude Sync
Comment=Sync your Claude Code environment across machines
Exec=$HOME/.local/bin/claude-sync.AppImage
Icon=$HOME/.local/share/icons/claude-sync.png
Type=Application
Categories=Development;Utility;
StartupWMClass=claude-sync
Terminal=false
ENTRY

update-desktop-database ~/.local/share/applications/ 2>/dev/null || true

echo ""
echo "Done! Press Super (Windows key) and search 'Claude Sync'."
echo ""
echo "To update later, just run this script again with the new .AppImage."
