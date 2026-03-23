# Velo — Command Palette for Windows

> Living doc. Update as steps complete.

---

## Stack

- **Language:** Rust (stable, edition 2024)
- **Win32 bindings:** `windows = "0.62.2"`
- **Rendering:** Direct2D + DirectWrite
- **Execution Engine:** `libs/velo_exec` (custom)
- **Config:** `serde` + `serde_yaml`
- **No:** eframe, egui, wgpu, or any GUI framework

---

## Project Structure

```text
velo/
├── Cargo.toml            — workspace root
├── src/
│   ├── main.rs           — entry point, DPI awareness
│   ├── window.rs         — Win32 message loop only
│   ├── command.rs        — YAML → Action mapping
│   ├── config.rs         — config + commands loader
│   ├── app/
│   │   ├── mod.rs        — re-exports
│   │   ├── state.rs      — AppState, InputMode
│   │   └── search.rs     — fuzzy search
│   └── renderer/
│       ├── mod.rs
│       ├── renderer.rs
│       ├── draw.rs
│       ├── layout.rs
│       ├── theme.rs
│       └── palette.rs
│
└── libs/
    └── velo_exec/
        ├── Cargo.toml
        └── src/
            ├── lib.rs
            ├── action.rs
            └── executor.rs
```

---

## Execution Architecture

### Core Principle

```text
UI builds Action → velo_exec runs → UI reacts
```

---

### Strict Boundaries

#### `velo_exec`

- Pure logic only
- No Win32
- No renderer
- No AppState
- No UI decisions

#### UI (`velo`)

- Builds `Action`
- Handles `InternalAction`
- Maps `ExecEvent → WindowAction`

---

## Action Model

Unified. No built-in vs user split.

```text
Action
├── Internal(InternalAction)
├── Launch { program, args }
├── OpenUrl { url }
├── Sequence(Vec<Step>)
├── Parallel(Vec<Action>)
```

---

### Step (for Sequence)

```text
Step
├── action: Action
├── wait: bool
├── stop_on_fail: bool
```

---

### InternalAction

```text
Quit
Hide
ReloadConfig
```

⚠️ Handled ONLY in UI

---

## Execution API

Inside `velo_exec`:

```rust
pub fn run(action: &Action) -> ExecEvent
```

---

### ExecEvent

```text
Done
Failed(String)
```

---

## Execution Flow

```text
Enter key
  ↓
Resolve command → Action
  ↓
if Internal → UI handles directly
  ↓
else → velo_exec::run()
  ↓
ExecEvent
  ↓
map → WindowAction
```

---

## WindowAction

```text
Quit
Hide
Nothing
```

Represents **UI response**, not execution logic.

---

## Input System

```text
Query
ArgInput { command, prompt_index, collected_args }
```

---

### Critical Rule

```text
All argument substitution happens BEFORE execution
```

Executor never sees `{0}`.

---

## Config Files

Located at:

```text
%APPDATA%\velo\
```

---

### config.yaml

```yaml
# velo configuration

# window_x: 960
# window_y: 400
```

- Auto-saved on drag
- Auto-generated if missing
- Invalid position → centered fallback

---

### commands.yaml

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

## Command Mapping

YAML → `Action`

```text
launch     → Action::Launch
open_url   → Action::OpenUrl
compound   → Action::Sequence
```

---

## Message Loop Flow

```text
WM_HOTKEY        → show window
WM_NCHITTEST     → title bar drag
WM_EXITSIZEMOVE  → save position
WM_PAINT         → draw UI
WM_CHAR          → input text
WM_KEYDOWN       → navigation + actions
WM_SETFOCUS      → focused = true
WM_KILLFOCUS     → hide window
WM_SIZE          → resize renderer
WM_CLOSE         → hide (not destroy)
WM_DESTROY       → quit app
```

---

## UI Layout

| Constant    | Value |
| ----------- | ----- |
| TITLE_BAR_H | 26.0  |
| QUERY_BAR_H | 48.0  |
| ROW_H       | 40.0  |
| DIVIDER_H   | 1.0   |
| PADDING_H   | 16.0  |
| MAX_ROWS    | 8     |

---

## Rendering

- Direct2D render target (cached)
- DirectWrite text layouts
- DPI-aware scaling
- Renderer dropped on hide

---

## Data Model

### CommandRef

```text
BuiltIn(usize) | User(usize)
```

---

### AppState

```text
query: String
arg_buffer: String
mode: InputMode
focused: bool
selected: usize
user_commands: Vec<UserCommand>
results: Vec<MatchedCommand>
config: AppConfig
```

---

## Resource Targets

| Metric     | Value  |
| ---------- | ------ |
| RAM idle   | ~3 MB  |
| RAM active | ~29 MB |
| CPU hidden | 0%     |
| Show delay | ~80ms  |

---

## Window Flags

```text
WS_POPUP
WS_EX_TOOLWINDOW
WS_EX_TOPMOST
```

---

## Notes & Gotchas

- Renderer must be cached
- `WM_SIZE` must call resize
- Logical vs physical pixels must be handled correctly
- `unsafe` required inside extern functions (Rust 2024)
- `WM_CLOSE` hides, does not destroy
- Executor must stay pure
- No UI logic inside execution layer
- No duplicate execution logic in UI
- All args resolved before execution

---

## Removed Architecture

```text
app/executor.rs ❌
run_builtin ❌
run_user ❌
```

Replaced by:

```text
libs/velo_exec ✅
```

---

## Built-in Commands

| Name            | Aliases      | Action                 |
| --------------- | ------------ | ---------------------- |
| Quit Velo       | exit, close  | Internal::Quit         |
| Reload Config   | refresh      | Internal::ReloadConfig |
| Open PowerShell | ps, terminal | Launch powershell      |
| Ping            | ping         | Launch ping {0}        |

---

# Current potentia; State

```text
velo = UI layer
velo_exec = execution engine
```

Clean separation. Scalable. No duplication.
