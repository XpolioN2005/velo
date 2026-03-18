# Velo — Command Palette for Windows

[![Rust](https://img.shields.io/badge/Rust-2024-blue.svg)]()
[![Platform](https://img.shields.io/badge/Platform-Windows-blue.svg)]()
[![Status](https://img.shields.io/badge/Status-Active%20Development-orange.svg)]()
[![License](https://img.shields.io/badge/License-MIT-green.svg)]()

A fast, native command palette for Windows built in Rust using Win32, Direct2D, and DirectWrite.

No frameworks. No Electron. No overhead.

---

## Features

- Native Win32 window (borderless, always-on-top)
- Direct2D rendering + DirectWrite text
- Fuzzy search with scoring and highlighting
- YAML-based command system
- Argument prompts with placeholder substitution
- Global hotkey activation (Ctrl + Alt + P)
- Window dragging + position persistence
- Low idle memory (~3MB)

---

## Tech Stack

- Rust (edition 2024)
- `windows = 0.62.2`
- Direct2D + DirectWrite
- serde + serde_yaml

---

## Project Structure

```

src/
├── main.rs
├── window.rs
├── command.rs
├── config.rs
├── app/
│ ├── mod.rs
│ ├── state.rs
│ ├── search.rs
│ └── executor.rs
└── renderer/
├── mod.rs
├── renderer.rs
├── draw.rs
├── layout.rs
├── theme.rs
└── palette.rs

```

---

## Usage

```bash
cargo run --release
```

Hotkey: **Ctrl + Alt + P**

---

## Configuration

Config files are stored in:

```
%APPDATA%\velo\
```

### config.yaml

```yaml
# window position (auto-managed)
# window_x: 960
# window_y: 400
```

- Automatically saved on window drag
- Falls back to center if invalid

---

### commands.yaml

```yaml
commands:
  - name: Example
    description: Example command
    aliases: [ex]
    action:
      type: launch
      program: notepad.exe
      args: []
```

---

## Performance

| Metric     | Value  |
| ---------- | ------ |
| RAM idle   | ~3 MB  |
| RAM active | ~29 MB |
| CPU hidden | 0%     |
| Show delay | ~80ms  |

---

## Architecture

- Strict separation:
  - Win32 layer (`window.rs`)
  - App logic (`app/`)
  - Rendering (`renderer/`)

- Event-driven (no polling)
- Renderer lifecycle managed (dropped on hide)
- Zero-copy command references

---

## Status

**Active development**

### Completed

- Window + message loop
- Rendering (Direct2D + DirectWrite)
- Input handling
- Command system
- Execution engine
- Fuzzy search
- Hotkey activation
- Window persistence

### In Progress

- System tray integration

### Planned

- Plugin system
- File/app indexing
- Usage-based ranking

---

## Goals

- Fast startup
- Minimal memory usage
- Native Windows experience
- Extensible command system

---

## License

MIT
