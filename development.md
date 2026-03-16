# Palette — Command Palette for Windows

> Living doc. Update as steps complete.

---

## Stack

- **Language:** Rust (stable toolchain)
- **Win32 bindings:** `windows` crate (latest, no pinned version)
- **Rendering:** Direct2D + DirectWrite
- **No:** eframe, egui, wgpu, or any GUI framework

---

## Project Structure

```
src/
├── main.rs     — entry point, DPI awareness, message loop
├── window.rs   — Win32 messages only, no draw logic
├── app.rs      — app state (query, focus, selected index, results)
├── command.rs  — Command structs, actions, ExecuteResult
├── config.rs   — TOML config loader (%APPDATA%\velo\commands.toml)
└── renderer/
    ├── mod.rs       — module declarations + re-exports only
    ├── renderer.rs  — Renderer struct (D2D factory, render target, DWrite)
    ├── draw.rs      — raw draw primitives (text, rect, outline, fill)
    ├── layout.rs    — region math
    ├── theme.rs     — all color constants
    └── palette.rs   — full UI assembly, calls draw + theme
```

---

## Cargo Features

| Step                            | Features                                                                                                      | Status |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------- | ------ |
| 1 — Win32 window + message loop | `Win32_UI_WindowsAndMessaging` `Win32_Foundation` `Win32_System_LibraryLoader` `Win32_Graphics_Gdi`           | ✅     |
| 2 — Direct2D render target      | `Win32_Graphics_Direct2D` `Win32_Graphics_Dxgi` `Win32_Graphics_Dxgi_Common` `Win32_Graphics_Direct2D_Common` | ✅     |
| 3 — DirectWrite text            | `Win32_Graphics_DirectWrite`                                                                                  | ✅     |
| 4 — Keyboard input              | _(covered by Step 1)_                                                                                         | ✅     |
| 5 — Static command list         | _(no new features)_                                                                                           | ✅     |
| 6 — Fuzzy search                | _(no new features)_                                                                                           | ⬜     |
| 7 — Hotkey + show/hide          | `Win32_UI_Input_KeyboardAndMouse` `Win32_UI_HiDpi`                                                            | ✅     |
| 7.5 — System tray icon          | `Win32_UI_Shell` _(TBD)_                                                                                      | ⬜     |
| 8 — Plugin API                  | _(TBD)_                                                                                                       | ⬜     |

---

## Message Loop Flow

```
WM_HOTKEY    → clear query, show window, steal focus (Ctrl+Alt+P)
WM_PAINT     → palette::draw_palette() — full UI redraw
WM_CHAR      → append char to query, resize, InvalidateRect
WM_KEYDOWN   → backspace, escape, enter, arrow keys
WM_SETFOCUS  → focused = true, InvalidateRect
WM_KILLFOCUS → focused = false, clear query, hide window
WM_SIZE      → renderer.resize(w, h) — keeps D2D in sync with window
WM_CLOSE     → clear query, hide window (process stays alive)
WM_DESTROY   → only on explicit quit — PostQuitMessage
```

---

## Resource Targets

- RAM idle: ~2–5 MB (release build, window hidden)
- CPU hidden: 0% (sleeps on GetMessage, no polling)
- Only wakes on: hotkey, keypress, paint

---

## Window Flags (important)

- `WS_POPUP` — borderless
- `WS_EX_TOOLWINDOW` — no taskbar entry
- `WS_EX_TOPMOST` — always on top
- `WS_EX_NOACTIVATE` — careful: conflicts with focus stealing on hotkey, manage explicitly

---

## Data Model

### `BuiltInCommand` — zero heap, lives in binary

```
name:        &'static str
description: &'static str
aliases:     &'static [&'static str]
category:    Category
action:      BuiltInAction
```

### `UserCommand` — heap, loaded from TOML at startup

```
name:        String
description: String
aliases:     Vec<String>
category:    Category
action:      UserAction
```

### `CommandRef` — lightweight index, no cloning

```
BuiltIn(usize) | User(usize)
```

### `ExecuteResult`

```
Quit | Hide | ReloadConfig | Nothing
```

### Config path

`%APPDATA%\velo\commands.toml` — missing file is silent, not an error

---

## Build Log

### Step 1 — Blank Win32 window + message loop ✅

- [x] `cargo new --bin velo`
- [x] Add windows crate with Step 1 features
- [x] `Win32_Graphics_Gdi` added (required for WNDCLASSEXW)
- [x] Window created with correct style flags
- [x] Message loop running
- [x] `WM_CLOSE` hides instead of killing — process stays alive
- [x] `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` — console in dev, silent in release

### Step 2 — Direct2D render target, clear color ✅

- [x] Create D2D1 factory
- [x] Create HWND render target (cached on Renderer struct)
- [x] Clear to `theme::BG` on WM_PAINT
- [x] Renderer stored on window via SetWindowLongPtrW / retrieved in wnd_proc

### Step 3 — DirectWrite text, draw query string ✅

- [x] Create DWrite factory
- [x] Create TextFormat struct in draw.rs (Segoe UI)
- [x] Draw "Search..." placeholder text on dark background
- [x] renderer/draw.rs split out for all draw primitives
- [x] renderer/renderer.rs owns Renderer struct

### Step 4 — Keyboard input ✅

- [x] WM_CHAR → append char to query string (max 100 chars)
- [x] WM_KEYDOWN → backspace, escape hides + clears, enter executes, arrow keys
- [x] WM_SETFOCUS / WM_KILLFOCUS → toggle focus, hide on lose focus
- [x] Border drawn around query bar only, changes color on focus/unfocus

### Step 5 — Command list + result rows ✅

- [x] `command.rs` — BuiltInCommand (&'static), UserCommand (heap), CommandRef, ExecuteResult
- [x] `config.rs` — TOML loader via serde, %APPDATA%\velo\commands.toml
- [x] `layout.rs` — query bar, row rects, name/desc split, window_height
- [x] Result rows drawn in palette.rs — name + description two-line layout
- [x] Dynamic window height via SetWindowPos + WM_SIZE → renderer.resize()
- [x] Divider between built-in and user command sections
- [x] Arrow keys move selected index, selection highlight drawn
- [x] Enter executes selected command via execute_selected() → ExecuteResult

### Step 6 — Fuzzy search wired up

- [ ] Replace substring match with fuzzy scoring
- [ ] Highlight matched characters in results

### Step 7 — Hotkey + show/hide ✅

- [x] RegisterHotKey — Ctrl+Alt+P
- [x] WM_HOTKEY → clear query, show, SetForegroundWindow, SetFocus
- [x] WM_KILLFOCUS → hide + clear query
- [x] DPI awareness — SetProcessDpiAwarenessContext at startup
- [x] ClearType text antialiasing
- [x] GetDesktopDpi → render target DPI, physical pixel size on init and resize

### Step 7.5 — System tray icon

- [ ] `Shell_NotifyIconW` — add icon to tray
- [ ] `WM_APP` custom message — receive tray mouse events
- [ ] Right-click context menu (Show / Quit)
- [ ] Quit destroys window and exits cleanly

### Step 8 — Plugin registration API

- [ ] TBD — design after Step 7.5 is solid

---

## Notes & Gotchas

- Direct2D render target must be **cached** on Renderer — never recreate per WM_PAINT or COM objects leak
- `WM_SIZE` must call `renderer.resize()` — without it D2D stretches content after SetWindowPos
- `pixelSize` in render target init must be physical pixels (logical × scale) — not logical pixels
- `resize()` receives logical pixels from WM_SIZE lparam — must multiply by scale before passing to D2D
- Fuzzy search must stay **synchronous and fast** for small lists; revisit if plugin count grows large
- `WM_KILLFOCUS` fires on own child windows too — handle carefully in Step 7.5+
- Edition 2024: `unsafe extern fn` bodies need explicit `unsafe {}` inside
- `WM_CLOSE` must hide, not destroy — `WM_DESTROY` is only for explicit quit via tray menu
- `Box::leak` used for LaunchProcess strings from config — small, lives for program lifetime, avoids lifetime params
- `results: Vec<CommandRef>` — indices not clones, rebuilt on every keystroke synchronously
