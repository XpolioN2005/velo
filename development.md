## Velo — Command Palette (Updated Architecture)

> Living doc. Reflects actual implementation.

---

# Execution Architecture

## Core Principle

```text
UI builds Steps → Executor → Platform → Process Engine → OS
```

---

## Strict Boundaries

### `velo_exec` (Execution Engine)

- Pure logic
- No Win32
- No UI
- No renderer
- No app state

Owns:

```text
✔ pipeline execution
✔ context system
✔ transforms
✔ process orchestration
✔ platform abstraction boundary
```

---

### `platform` (NEW Layer)

- OS-specific behavior ONLY
- Builds `Command`
- Handles shell behavior
- Handles URL opening semantics

---

### UI (`velo`)

- Builds `Vec<Step>`
- Supplies `Context.args`
- Handles user interaction
- Renders results
- Triggers execution
- Reacts to `StepResult`

---

# Execution Model

```text
Command = Vec<Step>

Step
├── action: Action
├── assign_to: Option<String>
```

---

# Action Model

```text
Action
├── LaunchProcess { program, args, mode, shell }
├── OpenUrl { url }
├── System(SystemActionId)
├── Transform(Transform)
```

---

# Transform System

```text
Transform
├── Regex { input?, pattern, group }
├── Split { input?, delimiter }
├── First
```

---

# Execution Flow

```text
Step 1 → result → ctx.last
Step 2 → uses ctx.last if input=None
Step N → continues chain
```

---

# Context

```text
ctx.args   → user input
ctx.vars   → variables
ctx.cwd    → working directory
ctx.last   → pipeline value
```

---

# Pipeline Rules

## Data Flow

```text
ctx.last is updated after EVERY step
```

---

## Input Resolution

```text
input = Some(...) → resolved
input = None      → ctx.last
```

---

## Placeholders

```text
{0}, {1}        → args
{var:name}      → variables
{last}          → previous output
```

---

# Execution Semantics

```text
success = false → STOP execution
error != None   → informational
```

No branching or recovery yet.

---

# Process Architecture (UPDATED)

## Key Change

```text
OLD:
Executor executed processes directly

NEW:
Executor → Platform → process.rs
```

---

## Responsibility Split

```text
Executor     → orchestration
Platform     → builds Command (OS-specific)
process.rs   → executes Command (OS-agnostic)
```

---

# Process Execution

## LaunchProcess

```text
program: String
args: Vec<String>
mode: ExecMode
shell: bool
```

---

## ExecMode

```text
FireForget
Capture
Stream
StreamMatch(regex)
```

---

## Shell Behavior (Now Platform-Controlled)

```text
shell = true  → handled inside Platform (e.g. cmd /C)
shell = false → direct execution
```

Executor does NOT know how shell works anymore.

---

# OpenUrl (UPDATED)

```text
Action::OpenUrl { url }
```

## Behavior

```text
Executor → Platform → build_command("start", ...)
```

- Fully platform-controlled
- No direct `cmd` usage in executor

---

# Process Engine (`process.rs`)

## Role

```text
Pure execution engine
```

Handles:

- spawn / output
- stdout + stderr merging
- streaming output
- regex matching on stream
- pipeline value preservation

---

# Example Pipeline (REAL)

```text
rg fn
→ "src/main.rs:10\n..."
→ Split("\n")
→ First
→ "src/main.rs:10"
→ Split(":")
→ First
→ "src/main.rs"
→ LaunchProcess(code)
```

---

# Execution API

```rust
Executor::run(&steps, &mut ctx) -> StepResult
```

---

# StepResult

```text
success: bool
value: Value
error: Option<String>
```

---

# Value Model

```text
Value
├── None
├── String
├── Bool
├── Number
├── List(Vec<Value>)
```

---

# YAML Mapping (IN PROGRESS)

```yaml
commands:
  - name: Search Code
    steps:
      - run:
          program: rg
          args: ["{0}"]
          mode: capture

      - transform:
          type: split
          delimiter: "\n"

      - transform:
          type: first

      - transform:
          type: split
          delimiter: ":"

      - transform:
          type: first
          assign_to: file

      - run:
          program: code
          args: ["{var:file}"]
          mode: fire_forget
          shell: true
```

---

# Command Mapping

```text
YAML → Vec<Step>
```

No nested execution structures.

---

# Removed Architecture

```text
Action::Sequence ❌
Action::Parallel ❌
wait flag ❌
stop_on_fail ❌
```

---

# Input Rule

```text
All placeholders SHOULD be resolved before execution
```

Executor still supports fallback resolution (temporary).

---

# Current Capabilities

```text
✔ Sequential pipelines
✔ Shared context (vars, args, cwd, last)
✔ Process execution (direct + shell)
✔ Platform abstraction (Windows implemented)
✔ Regex / Split / First transforms
✔ Variable assignment
✔ OpenUrl via platform
✔ Real pipelines (rg → parse → open)
✔ Streaming + regex matching
```

---

# Current Architecture

```text
UI (velo)
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
Executor: Complete
Platform Layer: Implemented (Windows)
Process Engine: Stable
Integration: In progress
```

---

# Known Design Issues

## 1. `shell: bool`

```text
Problem:
- weak abstraction
- platform-dependent meaning
- limits extensibility
```

### Planned Fix

```text
Replace with:
ExecMode::Shell
ExecMode::Direct
```

---

## 2. Limited Transform Set

Planned:

```text
- Replace
- Trim
- JSON parsing
```

---

## Direction

```text
From:
    Windows command palette

To:
    cross-platform execution pipeline engine
```
