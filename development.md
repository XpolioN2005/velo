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
├── main.rs     — entry point, message loop
├── window.rs   — Win32 messages only, no draw logic
├── app.rs      — app state (query, focus, selected index)
└── renderer/
    ├── mod.rs       — module declarations + re-exports only
    ├── renderer.rs  — Renderer struct (D2D factory, render target, DWrite)
    ├── draw.rs      — raw draw primitives (text, rect, outline)
    ├── layout.rs    — region math (Step 5+)
    ├── theme.rs     — all color constants
    └── palette.rs   — full UI assembly, calls draw + theme
```

---

## Cargo Features — Added Per Step

| Step                            | Features                                                                                                      | Status |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------- | ------ |
| 1 — Win32 window + message loop | `Win32_UI_WindowsAndMessaging` `Win32_Foundation` `Win32_System_LibraryLoader` `Win32_Graphics_Gdi`           | ✅     |
| 2 — Direct2D render target      | `Win32_Graphics_Direct2D` `Win32_Graphics_Dxgi` `Win32_Graphics_Dxgi_Common` `Win32_Graphics_Direct2D_Common` | ✅     |
| 3 — DirectWrite text            | `Win32_Graphics_DirectWrite`                                                                                  | ✅     |
| 4 — Keyboard input              | _(covered by Step 1)_                                                                                         | ✅     |
| 5 — Static command list         | _(no new features)_                                                                                           | ⬜     |
| 6 — Fuzzy search                | _(no new features)_                                                                                           | ⬜     |
| 7 — Hotkey + show/hide          | `Win32_UI_Input_KeyboardAndMouse`                                                                             | ⬜     |
| 7.5 — System tray icon          | `Win32_UI_Shell` _(TBD)_                                                                                      | ⬜     |
| 8 — Plugin API                  | _(TBD)_                                                                                                       | ⬜     |

> **Gotcha:** `WNDCLASSEXW` and `RegisterClassExW` require `Win32_Graphics_Gdi` — not pulled in by `Win32_UI_WindowsAndMessaging` alone.
> **Gotcha:** `D2D_SIZE_U` not `D2D1_SIZE_U` — lives in `Win32_Graphics_Direct2D_Common`.

---

## Message Loop Flow

```
WM_HOTKEY    → show window, steal focus
WM_PAINT     → palette::draw_palette() — full UI redraw
WM_CHAR      → append char to query, InvalidateRect
WM_KEYDOWN   → backspace, escape, enter, arrow keys
WM_SETFOCUS  → focused = true, InvalidateRect
WM_KILLFOCUS → focused = false, InvalidateRect
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

- [x] Add Step 2 cargo features
- [x] Create D2D1 factory
- [x] Create HWND render target (cached on Renderer struct)
- [x] Clear to `theme::BG` on WM_PAINT
- [x] Renderer stored on window via SetWindowLongPtrW / retrieved in wnd_proc

### Step 3 — DirectWrite text, draw query string ✅

- [x] Create DWrite factory
- [x] Create TextFormat struct in draw.rs (Segoe UI 14px)
- [x] Draw "Search..." placeholder text on dark background
- [x] renderer/draw.rs split out for all draw primitives
- [x] renderer/renderer.rs owns Renderer struct (mod.rs is init only)

### Step 4 — Keyboard input ✅

- [x] WM_CHAR → append char to query string
- [x] WM_KEYDOWN → backspace deletes, escape hides + clears, enter stubbed
- [x] WM_SETFOCUS / WM_KILLFOCUS → toggle focus state, repaint
- [x] Border color changes on focus/unfocus (theme::BORDER_FOCUS / BORDER_UNFOCUS)
- [x] AppState initialized with focused = true (window starts focused)
- [x] All colors moved to theme.rs, all paint logic moved to palette.rs

### Step 5 — Static command list, draw results

- [ ] Add command list to app.rs (name, description, action)
- [ ] layout.rs — calculate search bar + result row regions
- [ ] Draw result rows in palette.rs
- [ ] Dynamic window height based on result count via SetWindowPos
- [ ] Arrow keys move selected index up/down

### Step 6 — Fuzzy search wired up

- [ ] Filter command list on query string
- [ ] Highlight matched characters in results

### Step 7 — Hotkey + show/hide

- [ ] RegisterHotKey
- [ ] Show/hide window on WM_HOTKEY
- [ ] Hide on WM_KILLFOCUS

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
- Fuzzy search must stay **synchronous and fast** for small lists; revisit if plugin count grows large
- `WM_KILLFOCUS` fires on own child windows too — handle carefully in Step 7+
- Edition 2024: `unsafe extern fn` bodies are no longer implicitly unsafe — calls inside need their own `unsafe {}` block
- `WM_CLOSE` must hide, not destroy — `WM_DESTROY` is only for explicit quit via tray menu
- Brush creation in draw_text is per-call for now — optimize to cached brushes in theme.rs at Step 5
- `AppState::new()` sets `focused: true` — window starts focused, avoids wrong border color on first paint
