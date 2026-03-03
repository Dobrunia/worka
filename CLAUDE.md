# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## About the project

**Worka** — desktop time tracker for Windows 10 (Tauri v2 + Vue 3 + Rust + SQLite). Tracks active app, idle time, and input counts (no keylogging — counts only). All data stays local.

## Commands

```bash
# Run full app (Tauri + Vite together)
npm run dev

# If port 1420 is busy
npm run dev:clean

# Frontend only (no Rust)
npm run dev:vite

# Build release
npm run tauri build

# Frontend tests (watch mode)
npm run test

# Frontend tests (single run)
npm run test:run

# Backend tests
cd src-tauri && cargo test
```

## Architecture

The app is split into two layers that communicate via Tauri IPC commands:

**Frontend (`src/`)** — Vue 3 + TypeScript SPA loaded in Tauri webview:
- Four views: `TodayView`, `WeekView`, `TimelineView`, `SettingsView`
- Charts via `vue-chartjs` / `Chart.js`
- UI components from `dobruniaui-vue` (see rules below)

**Backend (`src-tauri/src/`)** — Rust binary running as a tray app:
- **Agent** — background `std::thread` with `Arc<AtomicBool>` stop flag; polls Windows API every N seconds and writes `activity_sample` rows
- **Input tracking** — global Windows hooks for keyboard/mouse counts → aggregated into `input_sample` buckets (no text saved)
- **Storage** — SQLite via `rusqlite`; tables: `app`, `activity_sample`, `input_sample`, `settings`
- **Tray** — Tauri tray API; close button hides window (doesn't quit); quit only via tray menu
- **Single-instance** — Windows Mutex `Local\WorkaSingleInstanceMutex`; second launch shows existing window and exits

**Process model (dev):**
```
npm run dev → tauri dev → Vite (port 1420) + cargo run (worka.exe)
```
Rust process must NOT start without Vite (white screen). Vite can run alone for UI-only work.

**Version management:** bump only `package.json` — `src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json` inherit it automatically.

## UI rules (dobruniaui-vue)

Priority order when building UI:
1. Use ready components: `DbrButton`, `DbrIconButton`, `DbrBadge`, `DbrCheckbox`, `DbrToggle`, `DbrInput`, `DbrAvatar`, `DbrTooltip`, `DbrLoader`, `DbrCard`, `DbrThemeToggle`
2. Then utility classes: `dbru-text-*`, `dbru-bg`, `dbru-surface`, `dbru-btn`, `dbru-btn--primary/ghost/danger`
3. Then CSS tokens: `--dbru-color-*`, `--dbru-space-*`, `--dbru-radius-*`, `--dbru-duration-*`
4. Custom styles only when nothing above fits

Docs: https://github.com/Dobrunia/DobruniaUI-vue

## Documentation files

| File | Purpose |
|------|---------|
| `functional.md` | Source of truth for WHAT features exist (MVP / L2 / L3 levels) |
| `stack.md` | Source of truth for WHICH technologies are used |
| `architecture.md` | HOW it's structured (components, DB schema, UI screens, behavior) |
| `RULES.md` | Rules for keeping docs in sync — follow when changing architecture |

**When adding features:** update `architecture.md` to match `functional.md` (do not add features not listed there unless marked Level 2/3). After architectural changes update `README.md` in marketing style (user benefits, not technical details).

## Key constraints

- Input tracking stores **counts only** — no keystrokes, no text, no screenshots
- Windows 10 target only (Windows API calls: `GetForegroundWindow`, `GetLastInputInfo`, `CreateMutexW`, etc.)
- Features not in `functional.md` must be explicitly tagged "Level 2" or "Level 3" in architecture docs
