## Velo — Build Log (Updated, Post Platform Refactor)

> Reflects current **actual architecture after executor + platform split**

---

# Phase 0 — Foundation (Unchanged)

### Win32 + Rendering Stack ✅

- Win32 window + message loop
- Direct2D renderer
- DirectWrite text system
- Input handling (keyboard + focus)
- Command palette UI
- Fuzzy search + ranking
- Global hotkey system
- Window persistence + config

---

# Phase 1 — Execution Engine (velo_exec)

## Step 8 — Core Engine ✅

- Step-based execution model
- Sequential pipeline execution
- Context system:
  - `ctx.args`
  - `ctx.vars`
  - `ctx.last`
  - `ctx.cwd`

---

## Step 8.1 — Process Execution (Refactored) ✅

### Major Change

```text
OLD:
Executor handled process execution directly

NEW:
Executor → Platform → process runner
```

### Final Split

```text
Platform     → builds Command (OS-specific)
process.rs   → executes Command (OS-agnostic)
Executor     → orchestrates pipeline
```

---

### Execution Modes ✅

- FireForget
- Capture
- Stream
- StreamMatch

All preserved after refactor.

---

## Step 8.2 — Pipeline System ✅

- Step chaining via `ctx.last`
- Implicit input when `input = None`
- Stable sequential execution
- Early exit on failure

---

## Step 8.3 — Variable System ✅

- `ctx.vars`
- `{var:name}` resolution
- assignment via `assign_to`

---

## Step 8.4 — Placeholder System ✅

- `{0}`, `{1}` → args
- `{var:name}` → variables
- `{last}` → pipeline chaining

---

## Step 8.5 — Transform Engine ✅

- Regex
- Split
- First
- ctx.last fallback

---

## Step 8.6 — OpenUrl (Refactored) ✅

```text
OLD:
cmd /C start inside executor

NEW:
Executor → Platform → build_command("start", ...)
```

Now fully platform-controlled.

---

## Step 8.7 — Error Handling ✅

- `StepResult`
- success flag
- error propagation
- pipeline stops on failure

---

## Step 8.8 — Testing ✅

Covers:

- Regex transforms
- Variables + placeholders
- ctx.last chaining
- Process execution (capture/stream)
- Real `rg` pipeline
- Split + First pipelines
- OpenUrl

---

# Phase 1.5 — Platform Abstraction (NEW) ✅

## Problem (Solved)

```text
- cmd /C hardcoded
- Windows-only logic inside executor
- poor portability
```

---

## Solution

### Platform Layer Introduced

```text
Executor → Platform → process.rs
```

---

## Platform Design

### Trait

```rust
pub trait Platform {
    fn build_command(
        &self,
        program: &str,
        args: &[String],
        shell: bool,
        ctx: &Context,
    ) -> Command;
}
```

---

## WindowsPlatform ✅

Handles:

- `cmd /C` wrapping
- `start` for URLs
- working directory
- shell vs direct execution

---

## Result

```text
✔ Executor is now OS-agnostic
✔ No cmd /C in core
✔ No process spawning in executor
✔ Clean boundary established
```

---

## process.rs (Final Role) ✅

Now acts as:

```text
Pure execution engine
```

Handles:

- spawn / output / streaming
- stdout + stderr merging
- regex stream matching
- pipeline value preservation

---

# Phase 2 — Integration (Current Focus)

## Step 9 — UI ↔ Executor Integration 🔄

### In Progress

- [ ] Replace legacy executor fully
- [ ] Map YAML → `Vec<Step>`
- [ ] Inject args into `ctx.args`
- [ ] Execute via `Executor::run`
- [ ] Handle `StepResult` in UI

---

## Step 9.1 — Command Mapping 🔄

- [ ] Convert YAML into pipeline steps
- [ ] Support transforms in config
- [ ] Support variable assignment

---

## Step 9.2 — ArgInput 🔄

- [ ] Capture user input
- [ ] Feed into `Context`
- [ ] Ensure correct placeholder resolution

---

# Phase 3 — Stabilization

- [ ] Remove legacy execution system
- [ ] Enforce strict UI / executor separation
- [ ] Validate real-world workflows:
  - search → open file
  - CLI pipelines
  - URL handling

---

# Current Architecture (IMPORTANT)

```text
UI Layer
    ↓
Executor (pipeline engine)
    ↓
Platform (OS abstraction)
    ↓
process.rs (execution engine)
    ↓
Operating System
```

---

# Current Status

```text
UI: Stable
Executor: Complete (pipeline + platform-aware)
Platform Layer: Implemented (Windows)
Process Engine: Stable
Integration: In progress
```

---

# Known Design Debt

## 1. `shell: bool`

```text
Problem:
- weak abstraction
- not expressive
- platform-dependent behavior hidden
```

### Planned Fix

```text
Replace with:
ExecMode::Shell
ExecMode::Direct
```

---

## 2. Limited Transform Set

Planned additions:

- Replace
- Trim
- JSON parsing

---

## Direction

```text
From:
    Windows-bound command launcher

To:
    cross-platform pipeline execution engine
    with clean OS abstraction
```

---
