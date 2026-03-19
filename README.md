# claude-sync

> Sync Claude Code environments (agents, memory, settings) across multiple machines using a private Git remote — like GitHub Desktop but for Claude Code.

![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows-informational)
![Built with](https://img.shields.io/badge/built%20with-Tauri%20v2%20%2B%20Rust-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

---

## What it syncs

| What | Where |
|------|-------|
| Agents | `~/.claude/agents/*.md` |
| Project memory | `~/.claude/projects/*/memory/**` |
| Settings | `~/.claude/settings.json` |

**Never touched (hardcoded exclusions):**
`.credentials.json` · `sessions/` · `history.jsonl` · `file-history/` · `cache/` · `backups/` · `telemetry/`

---

## Features

- **Automatic sync** — background timer (configurable interval, default 15 min)
- **Conflict detection** — SHA256 hash comparison per file, opens visual resolver on conflict
- **Side-by-side conflict resolver** — Mine / Theirs / Manual edit with Monaco editor
- **Agent manager** — browse, create, edit, delete agents with Monaco; 5 built-in templates
- **Memory browser** — tree view by project slug, inline editor, frontmatter-aware
- **Commit history** — see what changed, from which machine, with expandable diffs
- **System tray** — Sync Now, Open, Quit; notifications on sync events
- **Machine-local overrides** — specify settings keys that are never pushed/pulled (e.g. `settings.theme`)
- **Offline queue** — detects connectivity, queues changes, auto-flushes on reconnect
- **PAT stored in OS keychain** — never written to disk (libsecret on Linux, Credential Manager on Windows)
- **Zero git dependency** — uses libgit2 statically linked via `git2` crate

---

## Screenshots

```
┌─────────────────────────────────────────────────────────┐
│ claude-sync                              [─][□][✕]      │
├──────────┬──────────────────────────────────────────────┤
│          │  Dashboard                                   │
│ Dashboard│                                              │
│ Agents   │  Machine: Work Laptop    Last: 2 min ago     │
│ Memory   │  Remote:  user/claude    Pending: 0 files    │
│ History  │                                              │
│ Settings │  [↑ Sync Now]                                │
│          │                                              │
│ ● Online │  Pending Changes        Sync Status         │
└──────────┴──────────────────────────────────────────────┘
```

---

## Installation

### npm (recommended for developers)
```bash
npm install -g claude-sync
# or run without installing:
npx claude-sync
```

### Shell installer (Linux)
```bash
curl -fsSL https://raw.githubusercontent.com/user/claude-sync/main/scripts/install.sh | sh
```

### PowerShell installer (Windows)
```powershell
iwr -useb https://raw.githubusercontent.com/user/claude-sync/main/scripts/install.ps1 | iex
```

### GitHub Releases
Download `.deb`, `.AppImage`, or `.msi` from the [Releases page](https://github.com/user/claude-sync/releases).

---

## First-time setup

On first launch, claude-sync shows a 5-step wizard:

1. **Name this machine** — "Work Laptop", "Home PC", etc.
2. **Connect remote** — paste your GitHub repo URL + PAT
3. **Test connection** — verifies the token and repo are reachable
4. **Initial pull** — downloads any existing data
5. **Done** — auto-sync starts

### Creating a sync repo on GitHub

```bash
# Create a private repo (recommended: private to protect your data)
gh repo create my-claude-data --private --clone
```

Then paste the URL into the wizard. You need a PAT with `repo` scope.

---

## Conflict resolution

When both machines change the same file before syncing:

1. A conflict banner appears in the UI
2. Click **Resolve** on the conflicted file
3. Choose **Mine** (keep local), **Theirs** (use remote), or **Manual** (edit in Monaco)
4. The resolved version is committed on next sync

---

## Machine-local overrides

Settings keys listed in **Machine Overrides** are never pushed to or pulled from the remote. Useful for:
- `settings.theme` — dark/light preference per machine
- `settings.model` — different model per machine

Configure in **Settings → Machine**.

---

## Tech stack

| Layer | Choice |
|-------|--------|
| Backend | Rust + Tauri v2 |
| Git ops | `git2` crate (libgit2 vendored — no system git needed) |
| File watching | `notify` crate |
| Keychain | `keyring` crate (libsecret / Credential Manager) |
| Frontend | React + TypeScript + Vite |
| State | Zustand |
| UI | Tailwind CSS (dark theme: `#121212` bg, `#e53935` accent) |
| Editor | Monaco Editor |

---

## Building from source

### Prerequisites

**Linux:**
```bash
bash scripts/setup-dev.sh
# Installs: Rust, libwebkit2gtk-4.1-dev, libgtk-3-dev, libsecret-1-dev, librsvg2-dev, libappindicator3-dev
```

**Windows:** Install [Rust](https://rustup.rs), [Node.js](https://nodejs.org), and [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/).

### Dev mode
```bash
npm install
npm run tauri dev
```

### Production build
```bash
npm run tauri build
# Output: src-tauri/target/release/bundle/
```

---

## Repository structure

```
claude-sync/
├── src-tauri/src/
│   ├── claude/          # Claude file I/O (agents, memory, settings parsers)
│   ├── sync/            # Sync engine, conflict detection, file watcher
│   ├── git/             # git2 operations, PAT keychain auth, history
│   ├── commands/        # Tauri IPC commands (20+ functions)
│   └── tray.rs          # System tray
├── src/
│   ├── components/      # React UI (dashboard, agents, memory, history, settings, setup wizard)
│   ├── stores/          # Zustand state stores
│   ├── hooks/           # Tauri event listeners
│   └── lib/             # Typed IPC wrappers, utils
├── npm/                 # Platform packages for npm distribution
├── .github/workflows/   # Build + publish CI
└── scripts/             # Shell installers, dev setup
```

---

## IPC commands (Rust → React)

```
sync_now / sync_pull / sync_push / get_sync_status / get_pending_changes
list_agents / get_agent / save_agent / delete_agent / list_agent_templates / create_agent_from_template
list_memory_files / get_memory_file / save_memory_file / delete_memory_file / get_project_memories
get_commit_history / get_commit_diff / resolve_conflict
get_app_settings / save_app_settings / get_machine_config / save_machine_config / setup_remote / test_remote_connection
```

---

## Security

- PAT stored in **OS keychain** via `keyring` crate — never written to any file
- Credential files are **hardcoded exclusions** in Rust — cannot be accidentally synced regardless of config
- `.gitignore` is written by claude-sync on repo init — cannot be overridden by user pushes
- Sync repo should be **private** on GitHub

---

## License

MIT
