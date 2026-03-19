# claude-sync installer for Windows
# Usage: iwr -useb https://raw.githubusercontent.com/user/claude-sync/main/scripts/install.ps1 | iex

$ErrorActionPreference = "Stop"

$REPO = "user/claude-sync"
$INSTALL_DIR = "$env:LOCALAPPDATA\claude-sync\bin"
$BINARY = "claude-sync.exe"

Write-Host "Fetching latest release..."
$latest = (Invoke-RestMethod "https://api.github.com/repos/$REPO/releases/latest").tag_name

if (-not $latest) {
    Write-Error "Could not determine latest release"
    exit 1
}

Write-Host "Installing claude-sync $latest..."

$downloadUrl = "https://github.com/$REPO/releases/download/$latest/claude-sync-windows-x64.exe"

New-Item -ItemType Directory -Force -Path $INSTALL_DIR | Out-Null
Invoke-WebRequest -Uri $downloadUrl -OutFile "$INSTALL_DIR\$BINARY"

# Add to PATH if not already there
$currentPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
if ($currentPath -notlike "*$INSTALL_DIR*") {
    [System.Environment]::SetEnvironmentVariable(
        "PATH",
        "$currentPath;$INSTALL_DIR",
        "User"
    )
    Write-Host "Added $INSTALL_DIR to PATH"
    Write-Host "Restart your terminal to use claude-sync"
} else {
    Write-Host "Run: claude-sync"
}

Write-Host ""
Write-Host "claude-sync installed successfully!"
