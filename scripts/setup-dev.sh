#!/usr/bin/env bash
# Development setup script for claude-sync on Ubuntu/Debian
set -e

echo "Setting up claude-sync dev environment..."

# Install Rust if not present
if ! command -v cargo &>/dev/null && ! [ -f "$HOME/.cargo/bin/cargo" ]; then
  echo "Installing Rust..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
fi

echo "Installing system dependencies..."
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  libssl-dev \
  libdbus-1-dev \
  libsecret-1-dev \
  libgtk-3-dev \
  libglib2.0-dev

echo "Installing Node dependencies..."
npm install

echo ""
echo "Done! Run 'npm run tauri dev' to start in development mode."
