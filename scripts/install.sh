#!/usr/bin/env sh
# claude-sync installer for Linux
# Usage: curl -fsSL https://raw.githubusercontent.com/user/claude-sync/main/scripts/install.sh | sh

set -e

REPO="user/claude-sync"
INSTALL_DIR="${HOME}/.local/bin"
BINARY="claude-sync"

detect_arch() {
  case "$(uname -m)" in
    x86_64) echo "x64" ;;
    aarch64|arm64) echo "arm64" ;;
    *) echo "unsupported" ;;
  esac
}

main() {
  arch=$(detect_arch)
  if [ "$arch" = "unsupported" ]; then
    echo "Error: Unsupported architecture $(uname -m)"
    exit 1
  fi

  # Get latest release tag
  LATEST=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed 's/.*"tag_name": "\(.*\)".*/\1/')

  if [ -z "$LATEST" ]; then
    echo "Error: Could not determine latest release"
    exit 1
  fi

  echo "Installing claude-sync ${LATEST} for linux-${arch}..."

  DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${LATEST}/claude-sync-linux-${arch}"

  mkdir -p "${INSTALL_DIR}"
  curl -fsSL "${DOWNLOAD_URL}" -o "${INSTALL_DIR}/${BINARY}"
  chmod +x "${INSTALL_DIR}/${BINARY}"

  echo ""
  echo "Installed to ${INSTALL_DIR}/${BINARY}"

  # Check if in PATH
  case ":${PATH}:" in
    *":${INSTALL_DIR}:"*)
      echo "Run: claude-sync"
      ;;
    *)
      echo ""
      echo "Add to PATH by adding to ~/.bashrc or ~/.zshrc:"
      echo "  export PATH=\"\${HOME}/.local/bin:\${PATH}\""
      ;;
  esac
}

main
