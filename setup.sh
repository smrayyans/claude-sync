#!/bin/bash
# claude-sync installer
# Usage: curl -fsSL https://raw.githubusercontent.com/smrayyans/claude-sync/main/setup.sh | bash

set -e

REPO="smrayyans/claude-sync"
TMP="/tmp/claude-sync-install"

echo "==> Fetching latest release..."
DEB_URL=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
  | grep '"browser_download_url"' \
  | grep '\.deb"' \
  | head -1 \
  | cut -d'"' -f4)

if [ -z "$DEB_URL" ]; then
    echo "Error: Could not find .deb in latest release."
    exit 1
fi

mkdir -p "$TMP"
echo "==> Downloading $(basename "$DEB_URL")..."
curl -fsSL "$DEB_URL" -o "$TMP/claude-sync.deb"

echo "==> Installing (may ask for password)..."
sudo dpkg -i "$TMP/claude-sync.deb" || sudo apt-get install -f -y

rm -rf "$TMP"

echo ""
echo "  claude-sync installed!"
echo "  Search 'Claude Sync' in your app menu, or run: claude-sync"
echo ""
echo "  Uninstall:  sudo apt remove claude-sync"
echo "  Update:     re-run this script"
