# Velo — Build Log (Updated)

> Internal development log
> Reflects current real architecture and progress

---

## Phase 0 — Foundation

### Step 1 — Win32 Window + Message Loop ✅

- Window creation (`CreateWindowExW`)
- Message loop (`GetMessageW`)
- Hidden-on-close behavior
- Correct window styles:
  - `WS_POPUP`
  - `WS_EX_TOOLWINDOW`
  - `WS_EX_TOPMOST`

---

### Step 2 — Direct2D Setup ✅

- D2D factory created
- HWND render target initialized
- Clear background rendering
- Renderer stored in window state

---

### Step 3 — DirectWrite Text ✅

- Text rendering pipeline
- Query text + placeholder
- Text alignment + formatting
- ClearType enabled

---

### Step 4 — Input Handling ✅

- `WM_CHAR` text input

- `WM_KEYDOWN` controls:
  - Backspace
  - Escape
  - Enter
  - Arrow navigation

- Focus handling:
  - `WM_SETFOCUS`
  - `WM_KILLFOCUS`

---

### Step 5 — Command System (Initial) ✅

- Built-in commands
- User commands via YAML
- Result rendering
- Selection + highlighting
- Dynamic window sizing

---

### Step 5.5 — Execution (Legacy) ✅

- `app/executor.rs`
- Basic process launch
- URL opening
- Internal actions
- Early compound support

---

### Step 6 — Fuzzy Search ✅

- Scoring system
- Alias boost
- Match highlighting
- Sorted results

---

### Step 7 — Hotkey + Visibility ✅

- Global hotkey (`Ctrl + Alt + P`)
- Show / hide behavior
- DPI awareness
- Renderer lifecycle optimization

---

### Step 7.2 — Title Bar + Drag + Config ✅

- Custom title bar
- Native drag via `HTCAPTION`
- Window position persistence
- YAML config system

---

# Pivot — Execution Rewrite

### Problem

```text
Execution layer was tightly coupled to UI
Hard to extend
Control flow unclear
Not scalable
```

---

### Decision

```text
Extract execution into separate crate: velo_exec
```

---

### Outcome

```text
Execution is now:
- decoupled
- pipeline-based
- context-driven
```

---

# Phase 1 — New Execution Engine

### Step 8 — velo_exec (Core Engine) ✅

- [x] Created `libs/velo_exec`
- [x] Workspace integration
- [x] Defined Step-based execution model
- [x] Implemented Context system
- [x] Implemented sequential executor

---

### Step 8.1 — Process Execution ✅

- [x] Direct process execution
- [x] Shell execution (`cmd /C`)
- [x] Working directory support (`cwd`)
- [x] Modes:
  - FireForget
  - Capture
  - Stream
  - StreamMatch

---

### Step 8.2 — Pipeline System ✅

- [x] `ctx.last` chaining
- [x] Step-to-step data flow
- [x] Implicit input via `None`
- [x] Stable sequential execution

---

### Step 8.3 — Variable System ✅

- [x] `ctx.vars`
- [x] `{var:name}` resolution
- [x] assignment via `assign_to`
- [x] reusable pipeline values

---

### Step 8.4 — Placeholder System ✅

- [x] `{0}`, `{1}` args
- [x] `{var:name}`
- [x] `{last}`
- [x] integrated into resolver

---

### Step 8.5 — Transform Engine ✅

- [x] Regex transform
- [x] Split transform
- [x] First transform
- [x] ctx.last fallback behavior

---

### Step 8.6 — OpenUrl Action ✅

- [x] Added `Action::OpenUrl`
- [x] Uses `cmd /C start "" <url>`
- [x] Works with system default browser

---

### Step 8.7 — Error Handling ✅

- [x] `StepResult`
- [x] success flag
- [x] optional error message
- [x] failure stops execution

---

### Step 8.8 — Testing ✅

- [x] Unit tests for:
  - Regex
  - Variables
  - Args
  - ctx.last chaining
  - Process execution
  - Real `rg` pipeline
  - Split + First pipeline
  - OpenUrl

---

# Phase 2 — Integration (Current Focus)

### Step 9 — UI ↔ Executor Integration 🔄

- [ ] Replace old executor completely
- [ ] Map YAML → `Vec<Step>`
- [ ] Feed args into `Context`
- [ ] Trigger `Executor::run`
- [ ] Handle `StepResult` in UI

---

### Step 9.1 — Command Mapping Rewrite 🔄

- [ ] Replace Action-based mapping
- [ ] Build Step pipelines from YAML
- [ ] Support transforms in config

---

### Step 9.2 — ArgInput Integration 🔄

- [ ] Collect user input
- [ ] Inject into `ctx.args`
- [ ] Ensure full resolution before execution

---

# Phase 3 — Stabilization

- [ ] Remove all legacy execution code
- [ ] Ensure strict separation (UI vs executor)
- [ ] Validate real-world commands:
  - search → open file
  - url → browser
  - cli tools

---

# Future Work

## Execution Improvements

- [ ] Add more transforms:
  - Replace
  - Trim
  - JSON parse

- [ ] Improve error visibility

- [ ] Logging system

---

## Devlogs (Deferred)

Will include:

- Execution architecture evolution
- Pipeline design decisions
- Mistakes and rewrites
- Performance tuning

---

# Current Status

```text
UI: Stable
Execution Engine: Working (pipeline-based)
Integration: In progress
```

---

# Direction

```text
From:
    UI-driven execution

To:
    pipeline-based execution engine
    with clean separation
```
