#!/bin/bash
# claude-sync installer
# Usage: curl -fsSL https://raw.githubusercontent.com/smrayyans/claude-sync/main/setup.sh | bash

set -e

REPO="smrayyans/claude-sync"
DEB_FILE="/tmp/claude-sync.deb"

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

echo "==> Downloading $(basename "$DEB_URL")..."
curl -L -o "$DEB_FILE" "$DEB_URL"

# Verify it's actually a .deb
if ! file "$DEB_FILE" | grep -qi "debian"; then
    echo "Error: Downloaded file is not a valid .deb package."
    echo "URL was: $DEB_URL"
    echo "File type: $(file "$DEB_FILE")"
    rm -f "$DEB_FILE"
    exit 1
fi

echo "==> Installing (may ask for password)..."
sudo dpkg -i "$DEB_FILE" || sudo apt-get install -f -y
rm -f "$DEB_FILE"

echo ""
echo "  claude-sync installed!"
echo "  Search 'Claude Sync' in your app menu, or run: claude-sync"
echo ""
echo "  Uninstall:  sudo apt remove claude-sync"
echo "  Update:     re-run this script"
