# Velo — Command Palette for Windows

> Living doc. Reflects actual implementation.

---

## Execution Architecture

### Core Principle

```text
UI builds Steps → velo_exec executes → UI reacts
```

---

### Strict Boundaries

#### `velo_exec`

- Pure logic only
- No Win32
- No renderer
- No AppState
- No UI decisions
- Owns:
  - execution pipeline
  - context
  - transforms
  - process handling

#### UI (`velo`)

- Builds `Vec<Step>`
- Resolves all `{}` placeholders
- Provides `Context.args`
- Handles Internal actions
- Reacts to execution result

---

## Execution Model

```text
Command = Vec<Step>

Step
├── action: Action
├── assign_to: Option<String>
```

---

## Action Model (velo_exec)

```text
Action
├── LaunchProcess { program, args, mode, shell }
├── OpenUrl { url }
├── System(SystemActionId)
├── Transform(Transform)
```

---

## Transform

```text
Transform
├── Regex { input?, pattern, group }
├── Split { input?, delimiter }
├── First
```

---

## Execution Flow

```text
Step 1 → result → ctx.last
Step 2 → uses ctx.last (if input=None)
Step N → continues chain
```

---

## Context

```text
ctx.args   → user input (from UI)
ctx.vars   → assigned variables
ctx.cwd    → working directory
ctx.last   → pipeline value
```

---

## Pipeline Rules

### Data Flow

```text
ctx.last is ALWAYS updated after each step
```

---

### Input Resolution

```text
input = Some(...) → resolved string
input = None      → ctx.last
```

---

### Placeholders (executor-level)

```text
{0}, {1}        → args
{var:name}      → variables
{last}          → previous output
```

---

## Execution Semantics (CURRENT)

```text
success = false → STOP execution
error != None   → informational
```

No `critical`, no branching yet.

---

## Process Execution

```text
LaunchProcess
├── program: String
├── args: Vec<String>
├── mode: ExecMode
├── shell: bool
```

---

### ExecMode

```text
FireForget
Capture
Stream
StreamMatch(regex)
```

---

### Shell Mode (Windows)

```text
shell = true  → cmd /C ...
shell = false → direct execution
```

---

## OpenUrl

```text
Action::OpenUrl { url }
```

### Behavior

```text
cmd /C start "" <url>
```

- Uses default browser
- Uses shell internally

---

## Example Pipeline (REAL)

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

## Execution API (Actual)

```rust
Executor::run(&steps, &mut ctx) -> StepResult
```

---

### StepResult

```text
success: bool
value: Value
error: Option<String>
```

---

## Value Model

```text
Value
├── None
├── String
├── Bool
├── Number
├── List(Vec<Value>)
```

---

## YAML Mapping (WORK IN PROGRESS)

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

## Command Mapping

```text
YAML → Vec<Step>
```

NOT `Action::Sequence` anymore.

---

## Removed Architecture (OBSOLETE)

```text
Action::Sequence ❌
Action::Parallel ❌
wait flag ❌
stop_on_fail ❌
```

---

## Input Rule (IMPORTANT)

```text
All {0}, {var}, etc. resolved BEFORE execution
```

Executor should ideally only see final values
(but currently still supports resolution)

---

## Current System Capabilities

```text
✔ Sequential execution
✔ Shared context (vars, args, cwd, last)
✔ Process execution (shell + direct)
✔ Regex transform
✔ Split + First transforms
✔ Variable assignment
✔ OpenUrl support
✔ Real pipelines working (rg → parse → use)
```

---

## Current Position

```text
velo = UI layer
velo_exec = execution engine
```

---

## Next Direction

```text
1. Improve YAML → Step mapping
2. Add transforms (replace, trim)
3. Improve logging/debugging
4. Integrate cleanly with UI
5. THEN refine execution behavior (critical, etc.)
```
