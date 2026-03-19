# claude-sync

Sync Claude Code environments across machines using a private Git remote.

Keeps `agents/`, `skills/`, `projects/*/memory/`, `settings.json`, and **chat history** in sync — with a full GUI for browsing, editing, and resolving conflicts.

![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows-informational)
![Built with](https://img.shields.io/badge/Tauri%20v2%20%2B%20Rust-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

---

## Installation

Download from [Releases](https://github.com/smrayyans/claude-sync/releases).

### Linux

Download the `.AppImage`, then run the installer:

```bash
chmod +x install.sh && ./install.sh
```

This installs the app system-wide — search **"Claude Sync"** from your app menu.

> **Update:** download the new `.AppImage` and run `install.sh` again.
> **Uninstall:** `chmod +x uninstall.sh && ./uninstall.sh`

<details>
<summary>Alternative: .deb package (Debian/Ubuntu/Kali)</summary>

```bash
sudo dpkg -i claude-sync_*.deb
```
</details>

### Windows

Download and run `claude-sync_*_x64-setup.exe`. It installs to your Start Menu automatically.

<details>
<summary>Alternative: .msi installer</summary>

```
claude-sync_*_x64_en-US.msi
```
</details>

---

## Setup

1. **Create a private repo** on GitHub — name it `claude-data`, leave it empty
2. **Generate a PAT** — `GitHub → Settings → Developer settings → Personal access tokens (classic)` → `repo` scope
3. **Launch the app** — the setup wizard asks for machine name, repo URL, and PAT

---

## What gets synced

| Path | Description |
|------|-------------|
| `~/.claude/agents/*.md` | Custom subagent definitions |
| `~/.claude/skills/*.md` | Slash commands |
| `~/.claude/projects/*/memory/**` | Per-project memory |
| `~/.claude/projects/**/*.jsonl` | Chat history |
| `~/.claude/settings.json` | Claude Code settings |

**Excluded:** `.credentials.json`, `sessions/`, `history.jsonl`, `file-history/`, `cache/`, `backups/`, `telemetry/`

---

## Features

- **Dashboard** — Refresh / Pull / Push, status banners, device pull history, diagnose button
- **Agents** — Browse, create, edit with Monaco editor + 5 built-in templates
- **Memory** — Project tree with file browser and editor
- **Chats** — Browse all past Claude Code conversations by project
- **Sync Log** — Git commit history with diffs, conflict resolver (Mine / Theirs / Manual)
- **Settings** — Remote config, machine name, auto-sync interval, custom paths, machine-local overrides

---

<details>
<summary><strong>Advanced</strong></summary>

### Custom paths

Override default `~/.claude` directories in **Settings → Paths**. Stored in `~/.claude-sync/config.json`, applied via env vars at startup.

### Machine-local overrides

Keys listed in **Settings → Machine → Overrides** are stripped before push and ignored on pull. Useful for per-machine prefs like `settings.theme`.

### Config reference

```json
// ~/.claude-sync/config.json
{
  "machineId": "uuid-v4",
  "machineName": "Kali Laptop",
  "remoteUrl": "https://github.com/you/claude-data",
  "autoSyncInterval": 15,
  "machineOverrides": ["settings.theme"],
  "customPaths": { "claudeDir": "/custom/.claude" }
}
```

PAT is stored in OS keychain (libsecret / Credential Manager) — never on disk.

### Build from source

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev \
  libappindicator3-dev librsvg2-dev libsecret-1-dev patchelf
PATH="$HOME/.cargo/bin:$PATH" npm run tauri build
```

</details>

---

## Tech stack

Tauri v2 · Rust · git2 · React · TypeScript · Zustand · Monaco Editor · Tailwind CSS

---

MIT License
