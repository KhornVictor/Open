# 🚀 Open

[![Rust](https://img.shields.io/badge/rust-2024_edition-orange.svg)](https://www.rust-lang.org)
[![Version](https://img.shields.io/badge/version-1.0.0-blue.svg)](Cargo.toml)
[![Dependencies](https://img.shields.io/badge/dependencies-zero-brightgreen.svg)](Cargo.toml)
[![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-lightgrey.svg)]()

**Open** is a fast, lightweight, and zero-dependency terminal application and protocol launcher. It allows you to launch desktop applications, custom URI protocols (`obsidian://`, `discord://`, `spotify://`), and web URLs through an interactive menu or directly via command-line arguments.

---

## ✨ Features

- ⚡ **Instant Direct Launch**: Launch apps instantly using `open <name>` or any custom alias (e.g., `open obs`, `open chrome`).
- 🎨 **Polished Terminal UI**: Formatted Unicode box borders (`┌─┬─┐`), ANSI color coding, category tags, and responsive status indicators.
- 🔁 **Persistent Interactive Loop**: Run `open` without arguments to access an interactive menu that stays active until you choose to exit.
- 💡 **Smart Typo Tolerance**: Mistyped commands (e.g. `open obdian`) display clear error messaging with Levenshtein-based suggestions (`Did you mean: Obsidian?`).
- ⚙️ **Read-Only TOML Config**: Configure applications and aliases in [apps.toml](apps.toml) without needing to recompile the binary.
- 🪶 **Zero External Dependencies**: Built 100% on standard Rust library primitives for fast compile times and minimal binary size.

---

## 📦 Installation

### Option 1: PowerShell Installer (Windows)

Run the included [install.ps1](install.ps1) script or execute:

```shell
powershell -ExecutionPolicy Bypass -File .\install.ps1
```

Or via remote web script:
```shell
irm https://raw.githubusercontent.com/KhornVictor/Open/main/install.ps1 | iex
```

The installer installs `Open.exe` and `apps.toml` to `C:\Tool\Open` and adds it to your user `PATH`.

### Option 2: Build From Source

Make sure you have Rust installed (Rust 1.85+ / 2024 edition):

```powershell
git clone https://github.com/KhornVictor/Open.git
cd Open
cargo build --release
```

The compiled binary will be located at:
```
target\release\Open.exe
```

---

## 💻 Usage

### Interactive Mode
Run `open` without arguments to enter the interactive menu:

```shell
open
```

```text
  ┌────────────────────────────────────────────────────────────────────────┐
  │                      🚀 OPEN APPLICATION LAUNCHER                      │
  │               Fast, lightweight & modular quick launcher               │
  └────────────────────────────────────────────────────────────────────────┘

  ┌────┬──────────────────┬────────────────┬────────────────────────────┐
  │ #  │ Application      │ Category       │ Target / Scheme            │
  ├────┼──────────────────┼────────────────┼────────────────────────────┤
  │ 1  │ Obsidian         │ Productivity   │ obsidian://                │
  │ 2  │ Google Chrome    │ Internet       │ https://google.com         │
  │ 3  │ Discord          │ Communication  │ discord://                 │
  │ 4  │ Spotify          │ Media          │ spotify://                 │
  │ 5  │ VS Code          │ Development    │ vscode://                  │
  │ 6  │ GitHub           │ Development    │ https://github.com         │
  │ 7  │ Windows Terminal │ System         │ wt.exe                     │
  │ 8  │ Notepad          │ System         │ notepad.exe                │
  └────┴──────────────────┴────────────────┴────────────────────────────┘

  ────────────────────────────────────────────────────────────────────────
  [1-N] Select by #   [name] Type name/search   [c] Edit config   [r] Reload
  [0/q] Exit          Config: apps.toml
  ────────────────────────────────────────────────────────────────────────

  ❯ Choose application:
```

### Direct CLI Commands

| Command | Action |
| :--- | :--- |
| `open` | Open interactive menu |
| `open obsidian` | Launch Obsidian directly |
| `open obs` | Launch Obsidian using its alias |
| `open https://crates.io` | Open web URL directly in default browser |
| `open --list` (or `-l`) | Display table of configured applications |
| `open --config` (or `-c`) | Show path to active `apps.toml` |
| `open --help` (or `-h`) | Show help and usage guide |
| `open --version` (or `-v`) | Show current version |

### Typo Handling

If an application is not found in configuration:

```shell
open obdian
```

```text
  ✖ Application 'obdian' does not exist in configuration.
  💡 Did you mean: Obsidian?

  Available applications in configuration:
    • Obsidian (obsidian://)
    • Google Chrome (https://google.com)
    • Discord (discord://)
    ...

  Tip: Run open --list to see options, or open --config to add it to apps.toml.
```

---

## ⚙️ Configuration (`apps.toml`)

`Open` reads its configuration from `apps.toml` placed next to the binary or in the working directory.

### Example Configuration:

```toml
# Open - Application Launcher Configuration

[[app]]
name = "Obsidian"
aliases = ["obs", "notes"]
target = "obsidian://"
description = "Markdown knowledge base & personal wiki"
category = "Productivity"

[[app]]
name = "Google Chrome"
aliases = ["chrome", "browser", "google"]
target = "https://google.com"
description = "Default web browser"
category = "Internet"

[[app]]
name = "Discord"
aliases = ["dc"]
target = "discord://"
description = "Chat, voice & community servers"
category = "Communication"

[[app]]
name = "Spotify"
aliases = ["music", "sp"]
target = "spotify://"
description = "Music & podcast streaming player"
category = "Media"
```

Simple key-value format is also supported:

```toml
[apps]
Chrome = "https://google.com"
Discord = "discord://"
```

---

## 🏗️ Project Architecture

```
src/
├── main.rs          # Minimal application entry point and orchestrator
├── app.rs           # Application domain model, alias matching & Levenshtein distance
├── config.rs        # Read-only TOML parser & configuration file resolver
├── launcher.rs      # Cross-platform execution (cmd start, xdg-open, open)
├── cli.rs           # CLI argument handling, direct launch, and error suggestions
└── ui/
    ├── mod.rs       # Interactive menu loop and state management
    ├── ansi.rs      # Windows VT100 console setup, ANSI colors & styles
    └── components.rs# Box borders, formatted tables, headers & prompts
```

---

## 🧪 Testing

Run test suite:

```powershell
cargo test
```

Run linter:

```powershell
cargo clippy
```

---

## 📄 License

Licensed under the MIT License.
