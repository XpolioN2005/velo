# Velo — Build Log

> Internal development log.
> Detailed logs and devlogs will be written post-stabilization.

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

### Step 5.5 — Execution (Initial) ✅

- `app/executor.rs`
- Launch process
- Open URL
- Compound commands (basic)
- Internal actions (Quit, ReloadConfig)

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

# ⚠️ Pivot Point — Full Execution Rewrite

### Reason

```text
Execution layer became tightly coupled with UI and command system
Difficult to extend (sequence, parallel, argument flow)
Hard to reason about control flow
```

---

### Decision

```text
Rewrite execution as a separate library (velo_exec)
```

---

### Goals

- Decouple execution from UI
- Unify command model
- Enable:
  - Sequential execution
  - Parallel execution
  - Future extensibility

---

# Phase 1 — New Architecture (In Progress)

### Step 8 — Execution Engine Extraction 🔄

- [x] Created `libs/velo_exec`
- [x] Workspace integration
- [x] Defined new `Action` model
- [x] Introduced `ExecEvent`
- [ ] Removed old executor (`app/executor.rs`)
- [ ] Implement `Launch`
- [ ] Implement `OpenUrl`
- [ ] Implement `Sequence`
- [ ] Implement `Parallel`

---

### Step 9 — Integration Layer 🔄

- [ ] Map `Action → ExecEvent → WindowAction`
- [ ] Handle `InternalAction` in UI
- [ ] Replace old execution paths
- [ ] Validate ArgInput → Action pipeline

---

### Step 10 — Stabilization (Planned)

- [ ] Ensure no UI ↔ executor leakage
- [ ] Remove legacy code paths
- [ ] Clean command mapping
- [ ] Validate full execution flow

---

# Future Work (Deferred)

## Devlogs

Will include:

- Architecture decisions
- Execution model breakdown
- Rendering pipeline notes
- Trade-offs vs PowerToys

---

## Build Logs (Detailed)

Will include:

- Step-by-step evolution
- Mistakes and rewrites
- Performance tuning
- Edge case handling

---

# Current Status

```text
Core UI: Stable
Execution: Rewriting
Architecture: Improving
```

---

# Direction

```text
From:
    simple launcher with ad-hoc execution

To:
    structured system with clean execution engine
```

---

# Note

No detailed logs will be written until:

```text
execution system is stable and integrated
```

Focus remains on:

```text
correct architecture first
```
