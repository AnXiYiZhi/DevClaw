<div align="center">

# DevClaw

### The All-in-One Manager for AI Coding Tools

[![Version](https://img.shields.io/github/v/release/AnXiYiZhi/DevCLaw?color=blue&label=version)](https://github.com/AnXiYiZhi/DevCLaw/releases)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS-lightgrey.svg)](https://github.com/AnXiYiZhi/DevCLaw/releases)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-orange.svg)](https://tauri.app/)

### The Only Official Website: **[devclaw.cc.cd](https://devclaw.cc.cd)**

English | [中文](README.md) | [日本語](README_JA.md) | [Changelog](CHANGELOG.md)

</div>

## Introduction

DevClaw is a lightweight, blazing-fast desktop app built with Tauri 2 (Rust) that manages 7 AI coding tools from a single interface. It features 50+ built-in provider presets, local proxy with auto-failover, unified MCP/Skills/Prompts management, usage & cost tracking, and 100% local data storage.

## Supported Tools

| Tool | Description |
|------|-------------|
| Claude Code | Anthropic Official CLI |
| Claude Desktop | Anthropic Desktop App |
| Codex | OpenAI CLI |
| Gemini CLI | Google CLI |
| OpenCode | Open-source AI coding tool |
| OpenClaw | Open-source AI coding tool |
| Hermes Agent | Open-source AI Agent |

## Features

### Provider Management

- **7 tools, 50+ presets** — Official API, AWS Bedrock, community relays; copy your key and import with one click
- **Universal providers** — One config syncs to multiple tools (OpenCode, OpenClaw)
- System tray quick switch, drag-and-drop sorting, import/export

### Local Proxy & Failover

- **Built-in reverse proxy** — Format conversion, auto-failover, circuit breaker, provider health monitoring
- **App-level takeover** — Independently proxy Claude, Codex, or Gemini, down to individual providers

### MCP, Prompts & Skills

- **MCP Management** — Unified MCP server config across 4 apps with bidirectional sync
- **Prompts Management** — Markdown editor with cross-app sync (CLAUDE.md / AGENTS.md / GEMINI.md)
- **Skills Management** — One-click install from GitHub repos, custom repository support

### Usage & Cost Tracking

- **Usage Dashboard** — Track spending and tokens by model and provider, with custom pricing

### Session Manager

- Browse, search, and restore conversation history across all apps

### System & Platform

- **Cloud Sync** — Dropbox, OneDrive, iCloud, WebDAV
- **Deep Link** — Import configs via `devclaw://` URLs
- Dark / Light / System theme, auto-launch, auto-updater, i18n (en/zh/ja)

## Screenshots

|                  Main Interface                   |                  Add Provider                  |
| :-----------------------------------------------: | :--------------------------------------------: |
| ![Main Interface](assets/screenshots/main-en.png) | ![Add Provider](assets/screenshots/add-en.png) |

## Download

### System Requirements

- **Windows**: Windows 10 and above
- **macOS**: macOS 12 (Monterey) and above

### Installation

Visit the **[Download Page](https://devclaw.cc.cd/download)** or download from [GitHub Releases](https://github.com/AnXiYiZhi/DevCLaw/releases/latest).

**Windows**:
- `DevClaw-v{version}-Windows-Portable.zip` — Portable, extract and run
- `DevClaw-v{version}-Windows.msi` — Installer

**macOS**:
- `DevClaw-v{version}-macOS.dmg` — Recommended, drag to Applications

> macOS builds are notarized by Apple. You can install and open them directly.

## Quick Start

1. **Add Provider**: Click "Add Provider" → Choose a preset or create custom config
2. **Switch Provider**: Select provider → Click "Enable", or switch from the system tray
3. **Takes Effect**: Restart your terminal or CLI tool (Claude Code supports hot-switching)
4. **Back to Official**: Add an "Official Login" preset, restart, and follow the OAuth flow

### MCP, Prompts, Skills

- **MCP**: Click "MCP" → Add servers via templates or custom config
- **Prompts**: Click "Prompts" → Create presets → Activate to sync
- **Skills**: Click "Skills" → Browse GitHub repos → One-click install

## Development

### Requirements

- Node.js 18+
- pnpm 8+
- Rust 1.85+
- Tauri CLI 2.8+

### Commands

```bash
pnpm install          # Install dependencies
pnpm dev              # Dev mode (hot reload)
pnpm typecheck        # Type check
pnpm format           # Format code
pnpm test:unit        # Run unit tests
pnpm build            # Build application
```

### Tech Stack

**Frontend**: React 18 · TypeScript · Vite · TailwindCSS · TanStack Query · react-i18next · shadcn/ui

**Backend**: Tauri 2 · Rust · serde · tokio · SQLite

## Contributing

Issues and suggestions are welcome!

Before submitting PRs, please ensure:
- `pnpm typecheck`
- `pnpm format:check`
- `pnpm test:unit`

## License

MIT
