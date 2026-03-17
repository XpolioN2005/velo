markdown

# Velo — Command Palette for Windows

> Living doc. Update as steps complete.

---

## Stack

- **Language:** Rust (stable toolchain)
- **Win32 bindings:** `windows` crate (latest, no pinned version)
- **Rendering:** Direct2D + DirectWrite
- **No:** eframe, egui, wgpu, or any GUI framework

---

## Project Structure

src/
├── main.rs — entry point, DPI awareness, message loop
├── window.rs — Win32 messages only, no draw logic
├── app.rs — app state, InputMode, execution logic
├── command.rs — Command structs, unified Action enum, WindowAction
├── config.rs — YAML config loader (%APPDATA%\velo\commands.yaml)
└── renderer/
├── mod.rs — module declarations + re-exports only
├── renderer.rs — Renderer struct (D2D factory, render target, DWrite)
├── draw.rs — raw draw primitives (text, rect, outline, fill)
├── layout.rs — region math
├── theme.rs — all color constants
└── palette.rs — full UI assembly, calls draw + theme

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

WM_HOTKEY → clear query, show window, steal focus (Ctrl+Alt+P)
WM_PAINT → palette::draw_palette() — full UI redraw
WM_CHAR → append char to query or arg buffer, resize, InvalidateRect
WM_KEYDOWN → backspace, escape, enter, arrow keys — behavior branches on InputMode
WM_SETFOCUS → focused = true, InvalidateRect
WM_KILLFOCUS → focused = false, clear query, hide window
WM_SIZE → renderer.resize(w, h) — keeps D2D in sync with window
WM_CLOSE → clear query, hide window (process stays alive)
WM_DESTROY → only on explicit quit — PostQuitMessage

---

## Resource Targets

- RAM idle: ~3 MB (release build, window hidden — renderer dropped)
- RAM active: ~29 MB (renderer alive)
- CPU hidden: 0% (sleeps on GetMessage, no polling)
- Show delay: ~80ms (renderer recreate on hotkey)
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

name: &'static str
description: &'static str
aliases: &'static [&'static str]
action: Action

### `UserCommand` — heap, loaded from YAML at startup

name: String
description: String
aliases: Vec<String>
action: UserAction

### Unified `Action` enum — built-ins only

Internal(InternalAction) — Quit | Hide | ReloadConfig
LaunchProcess { program, args, prompts }
OpenUrl { url, prompts }
Compound(Vec<Action>)

### `UserAction` enum — mirrors Action with owned strings

LaunchProcess { program: String, args: Vec<String>, prompts: Vec<UserPrompt> }
OpenUrl { url: String, prompts: Vec<UserPrompt> }
Compound(Vec<UserAction>)

### `Prompt` / `UserPrompt`

label: &'static str / String — shown in query bar during ArgInput
optional: bool — if false, empty submission rejected

### Placeholder substitution

`{0}`, `{1}` etc in args/url replaced at execution time with collected arg values.

### `CommandRef` — lightweight index, no cloning

BuiltIn(usize) | User(usize)

### `WindowAction` — only what window.rs needs

Quit | Hide | Nothing

### `InputMode`

Query — normal search
ArgInput { command, prompt_index, collected_args } — collecting runtime args

`

### Config path

`%APPDATA%\velo\commands.yaml` — missing file is silent, not an error

---

## YAML Schema

yaml
commands:

- name: Open Notepad
  description: Launch Notepad
  aliases: [notepad, np]
  action:
  type: launch
  program: notepad.exe
  args: []

- name: Google
  description: Search Google
  aliases: [google, g]
  action:
  type: open_url
  url: "https://google.com/search?q={0}"
  prompts: - label: "Search:"
  optional: false

- name: Dev Setup
  description: Open editor and terminal
  aliases: [dev]
  action:
  type: compound
  steps: - type: launch
  program: code
  args: ["."] - type: launch
  program: wt.exe
  args: []
  `

---

## Build Log

### Step 1 — Blank Win32 window + message loop ✅

- [x] `cargo new --bin velo`
- [x] Add windows crate with Step 1 features
- [x] `Win32_Graphics_Gdi` added (required for WNDCLASSEXW)
- [x] Window created with correct style flags
- [x] Message loop running
- [x] `WM_CLOSE` hides instead of killing — process stays alive
- [x] `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`

### Step 2 — Direct2D render target, clear color ✅

- [x] Create D2D1 factory
- [x] Create HWND render target (cached on Renderer struct)
- [x] Clear to `theme::BG` on WM_PAINT
- [x] Renderer stored on window via SetWindowLongPtrW / retrieved in wnd_proc

### Step 3 — DirectWrite text, draw query string ✅

- [x] Create DWrite factory
- [x] TextFormat struct in draw.rs — left-aligned, vertically centered
- [x] TextFormat::new_right — right-aligned for descriptions
- [x] Draw "Search..." placeholder, query text, result rows
- [x] ClearType antialiasing enabled

### Step 4 — Keyboard input ✅

- [x] WM_CHAR → append char (max 100 chars), branches on InputMode
- [x] WM_KEYDOWN → backspace, escape, enter, arrow keys
- [x] WM_SETFOCUS / WM_KILLFOCUS → focus state, hide on lose focus
- [x] Border drawn around query bar only, changes color on focus/unfocus

### Step 5 — Command list + result rows ✅

- [x] `command.rs` — unified Action enum, BuiltInCommand, UserCommand, Prompt
- [x] `config.rs` — YAML loader via serde_yaml
- [x] `layout.rs` — query bar, row rects, name/desc same-line split (60/40), window_height
- [x] Result rows — name left, description right (dimmed), ellipsis trimming
- [x] Dynamic window height via SetWindowPos + WM_SIZE → renderer.resize()
- [x] Divider between built-in and user command sections
- [x] Arrow keys move selected index, selection highlight drawn
- [x] Renderer dropped on hide (~3MB idle), recreated on show (~80ms)

### Step 5.5 — Execution + ArgInput mode ✅

- [x] All execution logic in app.rs — window.rs only sees WindowAction
- [x] LaunchProcess — spawns detached process, hides
- [x] OpenUrl — cmd /c start, hides
- [x] Compound — fires all steps, hides
- [x] InternalAction::Quit — PostQuitMessage
- [x] InternalAction::ReloadConfig — stubbed
- [x] InputMode::ArgInput — prompt label shown in query bar, arg buffer collected
- [x] Placeholder substitution {0}, {1} at execution time
- [x] Backspace on empty arg buffer cancels back to Query mode
- [x] Escape in ArgInput cancels to Query mode, Escape in Query hides window
- [x] Window collapses to query bar height during ArgInput mode

### Step 6 — Fuzzy search wired up

- [ ] Replace substring match with fuzzy scoring
- [ ] Highlight matched characters in results

### Step 7 — Hotkey + show/hide ✅

- [x] RegisterHotKey — Ctrl+Alt+P
- [x] WM_HOTKEY → clear query, show, SetForegroundWindow, SetFocus
- [x] WM_KILLFOCUS → hide + clear query
- [x] DPI awareness — SetProcessDpiAwarenessContext at startup
- [x] GetDesktopDpi → render target DPI, physical pixel size on init and resize

### Step 7.5 — System tray icon

- [ ] `Shell_NotifyIconW` — add icon to tray
- [ ] `WM_APP` custom message — receive tray mouse events
- [ ] Right-click context menu (Show / Quit)
- [ ] Quit destroys window and exits cleanly

### Step 8 — Plugin registration API

- [ ] TBD — design after Step 7.5 is solid

---

## Built-in Commands

| Name            | Aliases                  | Action                                    |
| --------------- | ------------------------ | ----------------------------------------- |
| Quit Velo       | exit, close              | Internal::Quit                            |
| Reload Config   | refresh                  | Internal::ReloadConfig (stubbed)          |
| Open PowerShell | powershell, ps, terminal | LaunchProcess powershell.exe              |
| Ping            | ping                     | LaunchProcess powershell -NoExit ping {0} |

---

## Notes & Gotchas

- Direct2D render target must be **cached** on Renderer — never recreate per WM_PAINT or COM objects leak
- Renderer is `Option<Renderer>` on WindowState — dropped on hide, recreated on show
- `WM_SIZE` must call `renderer.resize()` — without it D2D stretches content after SetWindowPos
- `pixelSize` in render target init must be physical pixels (logical × scale)
- `resize()` receives logical pixels from WM_SIZE lparam — multiply by scale before passing to D2D
- `WM_KILLFOCUS` fires on own child windows too — handle carefully in Step 7.5+
- Edition 2024: `unsafe extern fn` bodies need explicit `unsafe {}` inside
- `WM_CLOSE` must hide, not destroy — `WM_DESTROY` is only for explicit quit via tray menu
- `results: Vec<CommandRef>` — indices not clones, rebuilt on every keystroke synchronously
- `substitute()` in command.rs handles {0}, {1} placeholder replacement at execution time
- Compound steps all fire in parallel (fire-and-forget) — sequential mode deferred to later
- Config silently returns empty vec on missing or malformed file — no crash on first run
