# claude-sync

Sync Claude Code environments across machines using a private Git remote.

![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows-informational)
![Built with](https://img.shields.io/badge/Tauri%20v2%20%2B%20Rust-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

---

## Why claude-sync?

Claude Code stores your agents, skills, memory, chat history, and settings locally under `~/.claude`. The moment you switch machines, you start from scratch.

claude-sync fixes this. It keeps everything in a private GitHub repo and gives you a GUI to browse, edit, and resolve conflicts — so your entire Claude Code environment travels with you.

- **No config drift** — one source of truth across all your machines
- **Full history** — every push is a Git commit; roll back anything
- **Private by default** — your data stays in your own GitHub repo

---

## Installation

Download from [Releases](https://github.com/smrayyans/claude-sync/releases).

### Linux (Debian / Ubuntu / Kali)

```bash
curl -fsSL https://raw.githubusercontent.com/smrayyans/claude-sync/main/setup.sh | bash
```

Then search **"Claude Sync"** in your app menu, or run `claude-sync` in terminal.

```bash
sudo apt remove claude-sync    # uninstall
```

> **Update:** run the install command again.

<details>
<summary>Alternative: AppImage (no root needed)</summary>

Download `.AppImage` + `install.sh` from [Releases](https://github.com/smrayyans/claude-sync/releases), then:

```bash
chmod +x install.sh && ./install.sh
```

Uninstall: `chmod +x uninstall.sh && ./uninstall.sh`

</details>

### Windows

Download and run `claude-sync_*_x64-setup.exe` from [Releases](https://github.com/smrayyans/claude-sync/releases). Installs to your Start Menu automatically.

<details>
<summary>Alternative: .msi installer</summary>

```
claude-sync_*_x64_en-US.msi
```

</details>

---

## Setup (3 steps)

1. **Create a private repo** on GitHub — name it `claude-data`, leave it empty
2. **Generate a PAT** — `GitHub → Settings → Developer settings → Personal access tokens (classic)` → `repo` scope
3. **Launch the app** — the setup wizard asks for machine name, repo URL, and PAT

---

## What gets synced

| Path | Description |
|---|---|
| `~/.claude/agents/*.md` | Custom subagent definitions |
| `~/.claude/skills/*.md` | Slash commands |
| `~/.claude/projects/*/memory/**` | Per-project memory |
| `~/.claude/projects/**/*.jsonl` | Chat history |
| `~/.claude/settings.json` | Claude Code settings |

**Excluded:** `.credentials.json`, `sessions/`, `history.jsonl`, `file-history/`, `cache/`, `backups/`, `telemetry/`

---

## Features

| Feature | Description |
|---|---|
| Dashboard | Refresh / Pull / Push, sync status, device pull history, diagnostics |
| Agents | Browse, create, and edit agent definitions with Monaco editor + 5 templates |
| Memory | Project tree with file browser and inline editor |
| Chats | Browse all past Claude Code conversations by project |
| Sync Log | Git commit history with diffs and a conflict resolver (Mine / Theirs / Manual) |
| Settings | Remote config, machine name, auto-sync interval, custom paths, machine-local overrides |

---

## Advanced

<details>
<summary>Custom paths</summary>

Override default `~/.claude` directories in **Settings → Paths**. Config is stored in `~/.claude-sync/config.json` and applied via env vars at startup.

</details>

<details>
<summary>Machine-local overrides</summary>

Keys listed in **Settings → Machine → Overrides** are stripped before push and ignored on pull. Useful for per-machine preferences like `settings.theme`.

</details>

<details>
<summary>Config reference</summary>

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

PAT is stored in the OS keychain (libsecret / Credential Manager) — never written to disk.

</details>

<details>
<summary>Build from source</summary>

```bash
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev \
  libappindicator3-dev librsvg2-dev libsecret-1-dev patchelf
PATH="$HOME/.cargo/bin:$PATH" npm run tauri build
```

</details>

---

## Tech Stack

| Layer | Technologies |
|---|---|
| Desktop shell | Tauri v2, Rust, git2 |
| Frontend | React, TypeScript, Zustand, Tailwind CSS |
| Editor | Monaco Editor |

---

## License

[MIT](LICENSE)
