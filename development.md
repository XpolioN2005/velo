# Velo — Command Palette for Windows

> Living doc. Update as steps complete.

---

## Stack

- **Language:** Rust (stable, edition 2024)
- **Win32 bindings:** `windows = "0.62.2"`
- **Rendering:** Direct2D + DirectWrite
- **Config:** `serde` + `serde_yaml`
- **No:** eframe, egui, wgpu, or any GUI framework

---

## Project Structure

```
src/
├── main.rs          — entry point, DPI awareness
├── window.rs        — Win32 message loop only, no draw logic
├── command.rs       — Command structs, unified Action enum, WindowAction
├── config.rs        — YAML loaders for config.yaml + commands.yaml
├── app/
│   ├── mod.rs       — routing only (re-exports AppState, InputMode, MatchedCommand)
│   ├── state.rs     — AppState, InputMode, all impl methods
│   ├── search.rs    — MatchedCommand, fuzzy_match, build_results
│   └── executor.rs  — run_builtin, run_user, open_url
└── renderer/
    ├── mod.rs       — module declarations + re-exports only
    ├── renderer.rs  — Renderer struct (D2D factory, render target, DWrite)
    ├── draw.rs      — raw draw primitives (text, rect, outline, fill)
    ├── layout.rs    — region math, all rects
    ├── theme.rs     — all color constants
    └── palette.rs   — full UI assembly, calls draw + theme
```

---

## Cargo Features

```toml
windows = { version = "0.62.2", features = [
    "Win32_UI_WindowsAndMessaging",
    "Win32_Foundation",
    "Win32_System_LibraryLoader",
    "Win32_Graphics_Gdi",
    "Win32_Graphics_Direct2D",
    "Win32_Graphics_Dxgi",
    "Win32_Graphics_Dxgi_Common",
    "Win32_Graphics_Direct2D_Common",
    "Win32_Graphics_DirectWrite",
    "Win32_UI_HiDpi",
    "Win32_UI_Input_KeyboardAndMouse",
    "Win32_Graphics_Dwm",
    "Win32_UI_Controls",
] }
```

---

## Config Files

Both live under `%APPDATA%\velo\`.

### `config.yaml`

App settings. Generated on first run if missing. Hand editing supported.

```yaml
# velo configuration

# window position — set automatically on drag, or override manually
# window_x: 960
# window_y: 400
```

- `window_x` / `window_y` are optional. Absent in generated default.
- Written automatically on `WM_EXITSIZEMOVE` (drag end) via `AppState::save_position()`.
- On startup, if saved position is off-screen (window center point off all monitors) → fall back to center.
- Full `serde_yaml` serialize on write (comments not preserved).

### `commands.yaml`

User-defined commands. Missing file is silent, not an error.

```yaml
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
      prompts:
        - label: "Search:"
          optional: false
```

---

## Message Loop Flow

```
WM_HOTKEY        → show window, steal focus (Ctrl+Alt+P)
WM_NCHITTEST     → return HTCAPTION if cursor in title bar region → native drag
WM_EXITSIZEMOVE  → drag ended, read GetWindowRect, call app.save_position()
WM_PAINT         → palette::draw_palette() — full UI redraw
WM_CHAR          → append char to query or arg buffer, resize, InvalidateRect
WM_KEYDOWN       → backspace, escape, enter, arrow keys — branches on InputMode
WM_SETFOCUS      → focused = true, InvalidateRect
WM_KILLFOCUS     → focused = false, clear query, hide window
WM_SIZE          → renderer.resize(w, h) — keeps D2D in sync with window
WM_CLOSE         → clear query, hide window (process stays alive)
WM_DESTROY       → only on explicit quit — PostQuitMessage
```

---

## UI Layout

All constants in `layout.rs`, unscaled (DPI scaling applied at render time).

| Constant      | Value | Notes                         |
| ------------- | ----- | ----------------------------- |
| `TITLE_BAR_H` | 26.0  | Always visible, incl ArgInput |
| `QUERY_BAR_H` | 48.0  |                               |
| `ROW_H`       | 40.0  |                               |
| `DIVIDER_H`   | 1.0   |                               |
| `PADDING_H`   | 16.0  |                               |
| `MAX_ROWS`    | 8     |                               |

Window minimum height = `TITLE_BAR_H + QUERY_BAR_H`. In `ArgInput` mode the results list collapses, title bar stays.

### Title Bar

- Drawn first, full width, `TITLE_BAR_BG` fill
- `"velo"` text left-aligned at `PADDING_H`, `TITLE_TEXT` color, `text_ui` format
- 1px bottom border in `DIVIDER` color separating it from query bar
- `WM_NCHITTEST` returns `HTCAPTION` for full title bar width — Windows handles drag natively

---

## Data Model

### `BuiltInCommand` — zero heap, lives in binary

```
name:        &'static str
description: &'static str
aliases:     &'static [&'static str]
action:      Action
```

### `UserCommand` — heap, loaded from YAML at startup

```
name:        String
description: String
aliases:     Vec<String>
action:      UserAction
```

### `Action` enum — built-ins only

```
Internal(InternalAction)   — Quit | Hide | ReloadConfig
LaunchProcess { program, args, prompts }
OpenUrl { url, prompts }
Compound(Vec<Action>)
```

### `UserAction` enum — mirrors Action with owned strings

```
LaunchProcess { program: String, args: Vec<String>, prompts: Vec<UserPrompt> }
OpenUrl { url: String, prompts: Vec<UserPrompt> }
Compound(Vec<UserAction>)
```

### `Prompt` / `UserPrompt`

```
label:    &'static str / String   — shown in query bar during ArgInput
optional: bool                    — if false, empty submission rejected
```

### `CommandRef` — lightweight index, no cloning

```
BuiltIn(usize) | User(usize)
```

### `WindowAction`

```
Quit | Hide | Nothing
```

### `InputMode`

```
Query
ArgInput { command, prompt_index, collected_args }
```

### `AppState`

```
query:         String
arg_buffer:    String
mode:          InputMode
focused:       bool
selected:      usize
user_commands: Vec<UserCommand>
results:       Vec<MatchedCommand>
config:        AppConfig          — owns window position, loaded + saved here
```

### `AppConfig`

```
window_x: Option<i32>
window_y: Option<i32>
```

Loaded in `AppState::new()`. Saved via `AppState::save_position(x, y)`. `window.rs` does not import config directly.

---

## Resource Targets

| Metric     | Value  | Notes                        |
| ---------- | ------ | ---------------------------- |
| RAM idle   | ~3 MB  | Renderer dropped on hide     |
| RAM active | ~29 MB | Renderer alive               |
| CPU hidden | 0%     | Sleeps on GetMessage         |
| Show delay | ~80ms  | Renderer recreated on hotkey |

---

## Window Flags

```
WS_POPUP          — borderless
WS_EX_TOOLWINDOW  — no taskbar entry
WS_EX_TOPMOST     — always on top
```

---

## Notes & Gotchas

- D2D render target must be **cached** — never recreate per `WM_PAINT` or COM objects leak
- `Renderer` is `Option<Renderer>` on `WindowState` — dropped on hide, recreated on show
- `WM_SIZE` must call `renderer.resize()` — without it D2D stretches content after `SetWindowPos`
- `pixelSize` in render target init must be physical pixels (logical × scale)
- `resize()` receives logical pixels from `WM_SIZE` lparam — multiply by scale before passing to D2D
- `D2D_POINT_2F` does not exist in windows-rs 0.62.2 — use local `#[repr(C)] struct Point2F` with `std::mem::transmute` for `DrawTextLayout`
- `HitTestTextRange` in 0.62.2: 6 args, takes `Option<&mut [DWRITE_HIT_TEST_METRICS]>` not raw pointer
- Edition 2024: `unsafe extern fn` bodies need explicit `unsafe {}` inside
- `WM_CLOSE` must hide, not destroy — `WM_DESTROY` only on explicit quit
- `WM_KILLFOCUS` fires on own child windows too — handle carefully in Step 7.5+
- `substitute()` in `command.rs` handles `{0}`, `{1}` placeholder replacement at execution time
- Compound steps all fire sequentially (fire-and-forget) — no return value collected
- Config silently returns defaults on missing or malformed file — no crash on first run
- `window.rs` reads position from `app.config` before `CreateWindowExW` by constructing `AppState::new()` first

---

## Build Log

### Step 1 — Blank Win32 window + message loop ✅

- [x] `cargo new --bin velo`
- [x] Add windows crate with Step 1 features
- [x] Window created with correct style flags
- [x] Message loop running
- [x] `WM_CLOSE` hides instead of killing — process stays alive
- [x] `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`

### Step 2 — Direct2D render target, clear color ✅

- [x] Create D2D1 factory
- [x] Create HWND render target (cached on Renderer struct)
- [x] Clear to `theme::BG` on `WM_PAINT`
- [x] Renderer stored via `SetWindowLongPtrW` / retrieved in `wnd_proc`

### Step 3 — DirectWrite text, draw query string ✅

- [x] Create DWrite factory
- [x] `TextFormat` struct in `draw.rs` — left-aligned, vertically centered
- [x] `TextFormat::new_right` — right-aligned for descriptions
- [x] Draw "Search..." placeholder, query text, result rows
- [x] ClearType antialiasing enabled

### Step 4 — Keyboard input ✅

- [x] `WM_CHAR` → append char (max 100), branches on `InputMode`
- [x] `WM_KEYDOWN` → backspace, escape, enter, arrow keys
- [x] `WM_SETFOCUS` / `WM_KILLFOCUS` → focus state, hide on lose focus
- [x] Border drawn around query bar, changes color on focus/unfocus

### Step 5 — Command list + result rows ✅

- [x] `command.rs` — unified `Action` enum, `BuiltInCommand`, `UserCommand`, `Prompt`
- [x] `config.rs` — YAML loader via serde_yaml
- [x] `layout.rs` — query bar, row rects, name/desc same-line split (60/40), `window_height`
- [x] Result rows — name left, description right (dimmed), ellipsis trimming
- [x] Dynamic window height via `SetWindowPos` + `WM_SIZE` → `renderer.resize()`
- [x] Divider between built-in and user command sections
- [x] Arrow keys move selected index, selection highlight drawn
- [x] Renderer dropped on hide (~3MB idle), recreated on show (~80ms)

### Step 5.5 — Execution + ArgInput mode ✅

- [x] All execution logic in `app/executor.rs` — `window.rs` only sees `WindowAction`
- [x] `LaunchProcess` — spawns detached process, hides
- [x] `OpenUrl` — `cmd /c start`, hides
- [x] `Compound` — fires all steps, hides
- [x] `InternalAction::Quit` — `PostQuitMessage`
- [x] `InternalAction::ReloadConfig` — stubbed
- [x] `InputMode::ArgInput` — prompt label shown in query bar, arg buffer collected
- [x] Placeholder substitution `{0}`, `{1}` at execution time
- [x] Backspace on empty arg buffer → back to Query mode
- [x] Escape in `ArgInput` → Query mode, Escape in Query → hide
- [x] Window collapses to query bar height during `ArgInput` (title bar stays)

### Step 6 — Fuzzy search ✅

- [x] `app/search.rs` — `fuzzy_match`, consecutive bonus, start-of-word bonus, alias boost +15
- [x] `MatchedCommand` — `cmd_ref`, `match_indices`, `score`
- [x] Results sorted by score descending
- [x] Match highlighting via `IDWriteTextLayout` + `HitTestTextRange` + `PushAxisAlignedClip`

### Step 7 — Hotkey + show/hide ✅

- [x] `RegisterHotKey` — Ctrl+Alt+P
- [x] `WM_HOTKEY` → show, `SetForegroundWindow`, `SetFocus`
- [x] `WM_KILLFOCUS` → hide + clear query
- [x] DPI awareness — `SetProcessDpiAwarenessContext` at startup
- [x] `GetDesktopDpi` → render target DPI, physical pixel size on init and resize

### Step 7.1 — app/ refactor ✅

- [x] `app.rs` split into `app/` directory
- [x] `mod.rs` — routing only, re-exports `AppState`, `InputMode`, `MatchedCommand`
- [x] `state.rs` — `AppState` + all `impl` methods
- [x] `search.rs` — `MatchedCommand`, `fuzzy_match`, `build_results`
- [x] `executor.rs` — `run_builtin`, `run_user`, `open_url`
- [x] All external `use crate::app::` imports unchanged

### Step 7.2 — Title bar + drag ✅

- [x] `TITLE_BAR_H = 26.0` in `layout.rs`, all rects shifted down
- [x] `TITLE_BAR_BG`, `TITLE_TEXT` added to `theme.rs`
- [x] Title bar drawn first in `palette.rs` — fill + "velo" label + 1px bottom border
- [x] `WM_NCHITTEST` in `window.rs` — returns `HTCAPTION` for full title bar width
- [x] `WM_EXITSIZEMOVE` → `app.save_position(x, y)` → `save_config`
- [x] `config.yaml` at `%APPDATA%\velo\config.yaml` — generated on first run
- [x] `AppConfig` owns position, lives on `AppState.config`
- [x] `window.rs` constructs `AppState::new()` before `CreateWindowExW` to read saved position

### Step 7.5 — System tray icon

- [ ] `Shell_NotifyIconW` — add icon to tray
- [ ] `WM_APP` custom message — receive tray mouse events
- [ ] Right-click context menu (Show / Quit)
- [ ] Quit destroys window and exits cleanly

### Step 8 — Plugin registration API

- [ ] TBD — design after Step 7.5 is solid

---

## Built-in Commands

| Name            | Aliases                  | Action                                      |
| --------------- | ------------------------ | ------------------------------------------- |
| Quit Velo       | exit, close              | `Internal::Quit`                            |
| Reload Config   | refresh                  | `Internal::ReloadConfig` (stubbed)          |
| Open PowerShell | powershell, ps, terminal | `LaunchProcess powershell.exe`              |
| Ping            | ping                     | `LaunchProcess powershell -NoExit ping {0}` |
