# Velo — Command Palette (Updated Architecture)

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

### `platform`

- OS-specific behavior ONLY
- Builds `Command`
- Handles shell execution
- Handles URL opening
- Owns all OS quirks

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

# Action Model (UPDATED)

```text
Action
├── LaunchProcess { program, args, mode }
├── Shell { command, mode }
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
ctx.last is updated after EVERY successful step
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

No branching or recovery.

---

# Process Architecture

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
```

- Direct execution
- No shell
- Strict argument handling

---

## Shell

```text
command: String
mode: ExecMode
```

- Full shell execution
- Supports pipes, redirects, chaining
- Platform-controlled

---

## ExecMode

```text
FireForget
Capture
Stream
StreamMatch(regex)
```

---

# OpenUrl

```text
Action::OpenUrl { url }
```

## Behavior

```text
Executor → Platform → build_open_url()
```

- Fully platform-controlled
- No OS logic in executor

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
- pipeline value propagation

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

# YAML Mapping (UPDATED)

```yaml
commands:
  - name: Search Code
    steps:
      - action:
          type: process
          program: "rg"
          args: ["{var:query}", "{var:path}", "--max-count", "100"]
          mode: capture

      - action:
          type: transform
          transform: split
          delimiter: "\n"

      - action:
          type: transform
          transform: first

      - action:
          type: transform
          transform: split
          delimiter: ":"

      - action:
          type: transform
          transform: first
        assign_to: "file"

      - action:
          type: process
          program: "code"
          args: ["{var:file}"]
          mode: fire_forget
```

---

# Alternative: Shell Command

For complex commands:

```yaml
- action:
    type: shell
    command: "rg {var:query} {var:path} --max-count 100 | head -n 1"
    mode: capture
```

---

# Command Mapping

```text
YAML → Vec<Step>
```

No nesting, no control flow.

---

# Removed Architecture

```text
Action::Sequence ❌
Action::Parallel ❌
wait flag ❌
stop_on_fail ❌
shell: bool ❌
```

---

# Input Rule

```text
All placeholders SHOULD be resolved before execution
```

Executor still supports fallback resolution.

---

# Current Capabilities

```text
✔ Sequential pipelines
✔ Shared context (vars, args, cwd, last)
✔ Direct process execution
✔ Shell execution (explicit)
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
Platform Layer: Clean (Windows implemented)
Process Engine: Stable
Integration: In progress
```

---

# Known Design Issues (UPDATED)

## 1. Argument Ergonomics

```text
Problem:
- Vec<String> is strict but not user-friendly
- YAML authoring feels verbose
```

### Direction

```text
Potential:
- shell-first commands
- or string → args parser layer
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

# Direction

```text
From:
    Windows command palette

To:
    cross-platform execution pipeline engine
```
