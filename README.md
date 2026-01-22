# Skill Manager 🧠

[English](./README.md) | [中文](./README.zh-CN.md)

**Skill Manager** is a powerful desktop application built with Tauri, Vue 3, and Rust, designed to manage skills for various AI agents (Anthropic, Claude Code, OpenCode, Cursor, and more).

## 📸 Screenshots

<p align="center">
  <img src="docs/screenshots/marketplace.png" alt="Marketplace" width="600" />
</p>
<p align="center"><em>Marketplace - Browse and discover skills from multiple sources</em></p>

<p align="center">
  <img src="docs/screenshots/installed.png" alt="Installed Skills" width="600" />
</p>
<p align="center"><em>Installed Skills - Manage your installed skills</em></p>

<p align="center">
  <img src="docs/screenshots/settings.png" alt="Settings" width="600" />
</p>
<p align="center"><em>Settings - Auto-detect installed AI coding agents</em></p>

## 🚀 Features

- **Centralized Marketplace**: Browse and discover skills from multiple official and community sources.
- **Multi-Agent Support**: Install skills to different AI agents with a single click.
- **Advanced Fetching**:
  - **Git-first**: Clones repositories directly for the most up-to-date code.
  - **HTTP Fallback**: Automatically falls back to ZIP downloads if Git is unavailable or requires authentication.
  - **Progress Tracking**: Real-time feedback on fetching progress and sources.
- **Local Management**: View, create, and uninstall skills from your local environment.
- **Persistent Caching**: Faster startup and offline access to marketplace metadata and source code.

## 📦 Sources

The marketplace aggregates skills from several high-quality sources:

- **Anthropic Official**: The core skill set for Anthropic agents.
- **Vercel Labs**: Experimental and cutting-edge agent skills.
- **Community Sources**: Various GitHub repositories containing useful templates and tools.

## 🛠️ Development

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/) (v1.75+)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

### Setup

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Tech Stack

- **Frontend**: Vue 3, Pinia, Vite, TypeScript, Lucide Icons.
- **Backend**: Rust, Tauri, Reqwest (HTTP), Tokios (Async), Zip Extraction.

## 🛠️ Troubleshooting (macOS)

If you encounter an "App is damaged" or "Cannot be opened" error on macOS after downloading, you can resolve it by removing the quarantine attribute:

```bash
sudo xattr -rd com.apple.quarantine /Applications/Skill\ Manager.app
```

## 📄 License

MIT
