# Skill Manager 🧠

[English](./README.md) | [中文](./README.zh-CN.md)

> **The App Store for your AI Agents (Cursor, Claude, etc.)**  
> **The easiest way to manage MCP servers locally.**

**Skill Manager** is a powerful desktop application built with Tauri, Vue 3, and Rust, designed to manage skills and MCP servers for various AI agents (Anthropic, Claude Code, OpenCode, Cursor, and more).

## 📸 Screenshots

<p align="center">
  <img src="docs/screenshots/marketplace.png" alt="Marketplace" width="600" />
</p>
<p align="center"><em>Marketplace - Browse and discover skills from multiple sources</em></p>

<p align="center">
  <img src="docs/screenshots/install-flow.gif" alt="Install Flow Demo" width="600" />
</p>
<p align="center"><em>Install Flow - One-click skill installation</em></p>

<p align="center">
  <img src="docs/screenshots/installed.png" alt="Installed Skills" width="600" />
</p>
<p align="center"><em>Installed Skills - Manage your installed skills</em></p>

<p align="center">
  <img src="docs/screenshots/settings.png" alt="Settings" width="600" />
</p>
<p align="center"><em>Settings - Auto-detect installed AI coding agents</em></p>

## 🚀 Features

- **The Agent App Store**: Browse and discover skills from multiple official and community sources (SkillsMP, Anthropic, etc.).
- **Smart Onboarding**: Automatically detects your installed agents (Cursor, VS Code, etc.) and offers one-click setup.
- **Improved Discovery**: 
  - Supports single-skill repositories (e.g., `BH-M87/why-what-how-skill`).
  - recursively discovers skills in monorepos (e.g., `anthropics/skills`).
- **Safety First**: 
  - Secure uninstallation with confirmation modals.
  - Prevents accidental deletion of critical skills.
- **Local MCP Manager** (Coming Soon):
  - Visual configuration for Model Context Protocol servers.
  - Switch between "Dev Mode" (Github/Postgres MCP) to "Writing Mode" instantly.
- **Multi-Agent Support**: Install skills to Cursor, VS Code, Claude, and more with a single click.
- **Advanced Fetching**:
  - **Git-first**: Clones repositories directly for the most up-to-date code.
  - **HTTP Fallback**: Automatically falls back to ZIP downloads if Git is unavailable.
- **Local Management**: View, create, and uninstall skills from your local environment.
- **Persistent Caching**: Faster startup and offline access.

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
