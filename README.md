# LinuxBrewer

A desktop GUI for [Homebrew (Linuxbrew)](https://brew.sh/) on Linux, built with Tauri v2, Vue 3, and TypeScript.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/barsazzar/LinuxBrewer/main/install.sh | sh
```

Requires `libwebkit2gtk-4.1` — if it's missing the script will tell you the exact command to install it for your distro (apt / dnf / pacman).

Or download the AppImage directly from the [Releases](https://github.com/barsazzar/LinuxBrewer/releases) page.

## Features

- **Installed packages** — browse all formulas and casks with version info; filter by kind or search by name
- **Upgradable packages** — see what's outdated; upgrade individually or all at once
- **Search & install** — search the Homebrew registry and install with one click
- **Taps** — list, add, and remove third-party taps
- **Real-time output** — long-running commands (install, upgrade, doctor…) stream output line-by-line in a live modal
- **Batch operations** — select multiple packages for bulk uninstall or upgrade
- **System tray** — minimize to tray with live outdated-package count badge
- **Brew version & path** — toolbar displays the detected brew version; sidebar shows the resolved binary path with a one-click copy button
- **Panel descriptions** — each panel has an info icon that reveals a brief description of what the panel does
- **Settings** — tabbed modal with Settings, About, and License tabs; supports custom brew path and language switching
- **Multilingual** — English / 中文, persisted across sessions
- **Keyboard shortcuts** — `Ctrl+R` refresh · `Ctrl+K` search · `Ctrl+F` filter · `Escape` closes modals

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

A custom path can be set in **Settings → Settings tab** if auto-detection fails.

## Author

Ding Li

## License

MIT License

Copyright (c) 2026 Ding Li

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
