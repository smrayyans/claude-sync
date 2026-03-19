# claude-sync

Sync Claude Code environments across machines using a private Git remote.

Keeps `agents/`, `skills/`, `projects/*/memory/`, `settings.json`, and **chat history** (`.jsonl` sessions) in sync — with a full GUI for browsing, editing, and resolving conflicts.

![Platform](https://img.shields.io/badge/platform-Linux%20%7C%20Windows-informational)
![Built with](https://img.shields.io/badge/Tauri%20v2%20%2B%20Rust-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

---

## What gets synced

| Path | Notes |
|------|-------|
| `~/.claude/agents/*.md` | Custom subagent definitions |
| `~/.claude/skills/*.md` | User-invocable `/slash` commands |
| `~/.claude/projects/*/memory/**` | Per-project memory files |
| `~/.claude/projects/**/*.jsonl` | Chat session history |
| `~/.claude/settings.json` | Claude Code settings |

**Hardcoded exclusions** (cannot be overridden): `.credentials.json`, `sessions/`, `history.jsonl`, `file-history/`, `cache/`, `backups/`, `telemetry/`

---

## Installation

### Option A — AppImage (Linux, no install)

Download the latest `.AppImage` from [Releases](https://github.com/smrayyans/claude-sync/releases), then:

```bash
chmod +x claude-sync_*.AppImage
./claude-sync_*.AppImage
```

### Option B — .deb package (Debian/Ubuntu/Kali)

```bash
sudo dpkg -i claude-sync_*.deb
claude-sync
```

### Option C — Build from source

```bash
# System deps (Debian/Ubuntu/Kali)
sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev \
  libappindicator3-dev librsvg2-dev libsecret-1-dev patchelf

# Rust + Node required
PATH="$HOME/.cargo/bin:$PATH" npm run tauri build

# Run the AppImage
./src-tauri/target/release/bundle/appimage/claude-sync_*.AppImage
```

> Always use `npm run tauri build`, not bare `cargo build` — the Tauri CLI embeds the frontend into the binary.

---

## Setup

### 1. Create a dedicated private repo on GitHub

Go to `github.com/new` — name it `claude-data`, set **Private**. Leave it completely empty (no README, no .gitignore). Do not reuse existing repos.

### 2. Generate a PAT

`github.com → Settings → Developer settings → Personal access tokens → Tokens (classic)` → generate with `repo` scope. Token starts with `ghp_`.

### 3. Run the setup wizard

Launch the app — on first run the setup wizard opens automatically.

Enter machine name, repo URL (`https://github.com/you/claude-data`), PAT → test connection → done.

---

## Features

**Dashboard** — Sync Now / Pull / Push buttons, last sync time, pending changes diff, online/offline status, device pull history (who pulled last and when)

**Agents** — Browse/create/edit agent `.md` files with Monaco editor, 5 built-in templates

**Memory** — Project tree → file browser → Monaco editor with frontmatter awareness

**Chats** — Browse all past Claude Code conversations grouped by project, search, read full transcripts

**Sync Log** — Git commit history with expandable diffs, conflict resolver (Mine / Theirs / Manual)

**Settings**
- *Remote* — repo URL + PAT + test button
- *Machine* — name, auto-sync interval, machine-local override keys
- *Paths* — override default `~/.claude` paths (see below)

---

## Custom paths

If Claude isn't installed at `~/.claude` or you want to point at a different directory:

**Settings → Paths** — set any of:

| Field | Default |
|-------|---------|
| Claude directory | `~/.claude` |
| Agents directory | `~/.claude/agents` |
| Skills directory | `~/.claude/skills` |
| Projects directory | `~/.claude/projects` |

Overrides are stored in `~/.claude-sync/config.json` and applied via env vars at startup. Takes effect after restart.

---

## Machine-local overrides

Keys listed in **Settings → Machine → Machine-local Overrides** are stripped before push and ignored on pull. Useful for per-machine preferences like `settings.theme` or `settings.fontSize`.

---

## Conflict resolution

When both machines change the same file before syncing, claude-sync detects it via SHA256 hash comparison (stored in `~/.claude-sync/hashes.json`) and opens the conflict resolver:

- **Mine** — discard remote, keep local
- **Theirs** — discard local, apply remote
- **Manual** — edit the resolved content in Monaco

---

## Chat history sync

Chat sessions (`.jsonl` files in `~/.claude/projects/`) are included in the sync. On a second machine, open the **Chats** tab to browse all past conversations pulled from the remote.

> Sessions can get large. If the sync repo exceeds ~10MB, consider excluding specific project slugs by adding their paths to machine overrides.

---

## Config file reference

`~/.claude-sync/config.json`:
```json
{
  "machineId": "uuid-v4",
  "machineName": "Kali Laptop",
  "remoteUrl": "https://github.com/you/claude-data",
  "autoSyncInterval": 15,
  "machineOverrides": ["settings.theme"],
  "customPaths": {
    "claudeDir": "/custom/path/.claude"
  }
}
```

PAT is stored in OS keychain (libsecret on Linux, Credential Manager on Windows) — never written to disk.

---

## Tech stack

Tauri v2 · Rust · `git2` (vendored libgit2) · `notify` · `keyring` · React · TypeScript · Zustand · Monaco Editor · Tailwind CSS

---

MIT License
