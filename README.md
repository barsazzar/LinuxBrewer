# LinuxBrewer

A desktop GUI for [Homebrew (Linuxbrew)](https://brew.sh/) on Linux, built with Tauri v2, Vue 3, and TypeScript.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/barsazzar/LinuxBrewer/main/install.sh | sh
```

Requires `libwebkit2gtk-4.1` — if it's missing the script will tell you the exact command to install it for your distro (apt / dnf / pacman).

Or download the AppImage directly from the [Releases](https://github.com/barsazzar/LinuxBrewer/releases) page.

## Uninstall

```bash
rm -f ~/.local/bin/LinuxBrewer ~/.local/bin/LinuxBrewer.appimage
```

## Features

- **Installed packages** — browse all formulas and casks with version info; filter by kind, search by name, or sort by name / kind / version
- **Upgradable packages** — see what's outdated with old → new version diff; upgrade individually or all at once
- **Package pinning** — pin packages to prevent them from being upgraded
- **Search & install** — search the Homebrew registry; results include version and description via `brew info`
- **Taps** — list, add, and remove third-party taps
- **Real-time output** — long-running commands (install, upgrade, doctor…) stream output line-by-line in a live modal with syntax highlighting
- **Cancellable operations** — cancel any in-progress command mid-stream
- **Batch operations** — select multiple packages for bulk uninstall or upgrade
- **System tray** — minimize to tray with live outdated-package count badge
- **Brew version & path** — toolbar displays the detected brew version; settings show the resolved binary path
- **Panel descriptions** — each panel has an info icon that reveals a brief description of what the panel does
- **Settings** — tabbed modal: Settings (custom brew path, language), Shortcuts, About, License
- **Multilingual** — English / 中文, persisted across sessions
- **Keyboard shortcuts** — `Ctrl+R` refresh · `Ctrl+K` search · `Ctrl+F` filter · `Ctrl+,` settings · `Escape` closes modals

## Tech Stack

| Layer    | Tech                                        |
| -------- | ------------------------------------------- |
| Shell    | [Tauri v2](https://tauri.app/) (Rust)       |
| Frontend | Vue 3 `<script setup>` + TypeScript         |
| Build    | Vite                                        |
| Styling  | Plain CSS (global, no component scoping)    |
| Icons    | [Lucide](https://lucide.dev/) (lucide-vue-next) |
| State    | Module-level `ref()` singleton (no Pinia)   |
| i18n     | Reactive `computed` translation map         |

## Project Structure

```
src/
├── i18n/index.ts          # Language definitions (en / zh) + reactive t
├── store/
│   ├── brew.ts            # Re-export hub (single import point for components)
│   ├── packages.ts        # Installed/outdated packages, install/uninstall/upgrade logic
│   ├── search.ts          # Search state + doSearch / enrichment
│   ├── taps.ts            # Taps list + add/remove
│   ├── settings.ts        # Custom brew path + language settings
│   └── ui.ts              # UI state (toasts, confirm modal, detail modal)
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
├── types.rs     # Rust structs + ok/err helpers + BrewState cache
├── brew.rs      # Homebrew helpers + sync Tauri commands
├── stream.rs    # brew_run_stream (async, tokio) + cancellation registry
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
