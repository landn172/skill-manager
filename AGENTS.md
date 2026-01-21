# AGENTS.md

**Skill Manager**: Desktop app for managing AI agent skills (Anthropic, Claude, OpenCode, VS Code).
**Stack**: Vue 3, TypeScript, Rust (Tauri 2.0), SQLite.

## Structure

- `src/`: Vue Frontend (Pages: Marketplace, Installed, Settings)
- `src-tauri/src/`: Rust Backend (Commands: `installer`, `marketplace`, `skillsmp`)
- `.github/workflows/`: CI/CD

## Key Commands

- `detect_agents`: Find installed agents.
- `fetch_marketplace_skills` / `fetch_skillsmp_skills`: Get skills.
- `install_skill` / `uninstall_skill`: Manage skills.

## Dev

```bash
npm install && npm run tauri dev  # Run
npm run tauri build               # Build
npm run test                      # Test
```

## Changelog

- **2026-01-20**: Added CI/CD Release flow, Install Progress UI, Icon updates.
- **2026-01-19**: Cloudflare bypass for SkillsMP.
- **2026-01-15**: VS Code support, Cache fixes, SkillsMP API.

## Notes

- **Tauri v2** syntax.
- **Rust 2021** edition.
- **Sources**: Git-first, HTTP fallback.
