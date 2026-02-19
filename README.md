# Brew Manager

A desktop GUI for [Homebrew (Linuxbrew)](https://brew.sh/) on Linux, built with Tauri v2, Vue 3, and TypeScript.

## Features

- **Installed packages** — browse all formulas and casks with version info; filter by kind or search by name
- **Upgradable packages** — see what's outdated; upgrade individually or all at once
- **Search & install** — search the Homebrew registry and install with one click
- **Taps** — list, add, and remove taps
- **Real-time output** — long-running commands (install, upgrade, doctor…) stream output line-by-line in a live modal
- **Batch operations** — select multiple packages for bulk uninstall or upgrade
- **System tray** — minimize to tray; click to restore
- **Multilingual** — English / 中文, persisted across sessions
- **Keyboard shortcuts** — `Ctrl+F` focuses the package filter input; `Escape` closes modals

## Tech Stack

| Layer    | Tech                                      |
| -------- | ----------------------------------------- |
| Shell    | [Tauri v2](https://tauri.app/) (Rust)     |
| Frontend | Vue 3 `<script setup>` + TypeScript       |
| Build    | Vite                                      |
| Styling  | Plain CSS (global, no component scoping)  |
| State    | Module-level `ref()` singleton (no Pinia) |
| i18n     | Reactive `computed` translation map       |

## Project Structure

```
src/
├── i18n/index.ts          # Language definitions (en / zh) + reactive t
├── store/brew.ts          # All reactive state + business logic
├── types.ts               # Shared TypeScript interfaces
├── assets/global.css      # All styles
├── App.vue                # Thin layout orchestrator
└── components/
    ├── AppToolbar.vue
    ├── SearchPanel.vue
    ├── InstalledPanel.vue
    ├── UpgradablePanel.vue
    ├── TapsPanel.vue
    ├── LogsDrawer.vue
    ├── BatchBar.vue
    └── modals/
        ├── ConfirmModal.vue
        ├── DetailModal.vue
        ├── AddTapModal.vue
        └── SettingsModal.vue

src-tauri/src/
├── lib.rs       # Entry point + command registration
├── types.rs     # Rust structs + ok/err helpers
├── brew.rs      # Homebrew helpers + sync Tauri commands
├── stream.rs    # brew_run_stream (async, tokio)
└── tray.rs      # System tray setup + update_tray command
```

## Development

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

Requires [Rust](https://rustup.rs/) and the [Tauri CLI prerequisites](https://tauri.app/start/prerequisites/).

## Homebrew Detection

The app auto-detects `brew` from the standard Linuxbrew and macOS paths:

- `/home/linuxbrew/.linuxbrew/bin/brew`
- `/linuxbrew/.linuxbrew/bin/brew`
- `/usr/local/bin/brew`
- `/opt/homebrew/bin/brew`
- `brew` (from `$PATH`)

A custom path can be set in **Settings** if auto-detection fails.
