# Velo — Execution Engine (`velo_exec`) Documentation

> Fully decoupled, OS-agnostic, pipeline-first execution engine.

---

## 1. Motivation

The legacy execution system had several issues:

- Coupled tightly with UI and Win32 → not testable independently.
- Sequential compound step system was restrictive:
  - Steps could not reference outputs from earlier steps easily.
  - `.output()` blocking calls froze GUI applications.

- Global `shell` flag → mixed OS logic with executor → impossible to generalize for cross-platform.
- YAML schema was limited → complex pipelines were difficult to express.
- No clean separation between orchestration, platform logic, and process execution.

**Solution:** Rebuild the execution engine as a separate, OS-agnostic crate (`velo_exec`) with strict boundaries:

- UI handles user input & rendering only.
- Executor handles orchestration, transforms, pipeline execution.
- Platform layer handles OS-specific command execution.
- `process.rs` handles pure process spawning, streaming, and output handling.

---

## 2. Architecture Overview

```text
UI Layer (velo)
    ↓ builds Vec<Step>, supplies ctx.args
Executor (velo_exec)
    ↓ orchestrates sequential pipeline
Platform Layer
    ↓ builds OS-specific commands
Process Engine (process.rs)
    ↓ executes commands at OS level
Operating System
```

### **Data Flow Diagram (Conceptual)**

<img src="https://i.ibb.co/c0TFYkk/mermaid-diagram-2026-03-28-165036.png" alt="mermaid-diagram-2026-03-28-165036" border="2">

### Layer Responsibilities

| Layer          | Responsibilities                                                                        |
| -------------- | --------------------------------------------------------------------------------------- |
| UI             | Input, placeholder resolution, YAML → Steps mapping, rendering, receiving StepResult    |
| Executor       | Pipeline orchestration, context, transform system, sequential execution, error handling |
| Platform       | OS-specific command construction (`LaunchProcess`, `Shell`, `OpenUrl`)                  |
| Process Engine | Pure process execution, streaming stdout/stderr, regex matching, value propagation      |

---

## 3. Core Concepts

### Step

```rust
struct Step {
    action: Action,
    assign_to: Option<String>,
}
```

- `action` → what to execute (process, shell, URL, transform, system)
- `assign_to` → optional variable name to store step output in `ctx.vars`

### Action

```rust
enum Action {
    LaunchProcess { program: String, args: Vec<String>, mode: ExecMode },
    Shell { command: String, mode: ExecMode },
    OpenUrl { url: String },
    System(String),
    Transform(Transform),
}
```

- **LaunchProcess:** direct process execution, strict arguments, no shell.
- **Shell:** full shell execution, supports pipes, redirects, chaining.
- **OpenUrl:** OS-agnostic URL opening.
- **Transform:** regex, split, first, or custom transforms on step output.
- **System:** OS-level and internal actions.

---

## 4. Transform System

```rust
enum Transform {
    Regex { input: Option<String>, pattern: String, group: usize },
    Split { input: Option<String>, delimiter: String },
    First,
    // Planned: Replace, Trim, JSON parsing
}
```

- `input: Option<String>` → fallback to `ctx.last` if `None`.
- Supports chaining of transforms.
- Updates `ctx.last` after execution.

---

## 5. Context System

```rust
struct Context {
    args: Vec<String>,    // User-supplied input arguments
    vars: HashMap<String, Value>, // Named variables
    last: Value,          // Output of last step in the pipeline
    cwd: PathBuf,         // Optional Working directory
}
```

- Used for placeholder resolution and pipeline chaining.
- Updated automatically after each successful step.

### Placeholders

| Placeholder               | Source                                        |
| ------------------------- | --------------------------------------------- |
| `{0}`, `{1}`, ... , `{n}` | Positional args from `ctx.args`               |
| `{var:name}`              | Variable in `ctx.vars`                        |
| `{last}`                  | Output of previous pipeline step (`ctx.last`) |

---

## 6. Pipeline Rules

- **Sequential execution:** Step N executes after Step N-1.
- **Input resolution:**
  - `Some(input)` → step uses provided input
  - `None` → fallback to `ctx.last`

- **Error handling:** `success = false` stops pipeline; `error` is informational.
  > Planned `Soft` and `Hard` error later

---

## 7. Execution Modes (`ExecMode`)

| Mode        | Description                                     |
| ----------- | ----------------------------------------------- |
| FireForget  | Launch process without waiting/capturing output |
| Capture     | Capture stdout/stderr into `StepResult.value`   |
| Stream      | Stream output in real-time                      |
| StreamMatch | Stream output and apply regex matches           |

---

## 8. StepResult & Value

```rust
struct StepResult {
    success: bool,
    value: Value,
    error: Option<String>,
}

enum Value {
    None,
    String(String),
    Bool(bool),
    Number(f64),
    List(Vec<Value>),
}
```

- `StepResult` returned after every step.
- `Value` is the typed output propagated via `ctx.last`.

---

## 9. Executor API

```rust
Executor::run(steps: &[Step], ctx: &mut Context) -> StepResult
```

- Runs a sequential pipeline.
- Updates `ctx.last` and `ctx.vars` as defined by `assign_to`.
- Applies transforms and placeholder resolution.
- Returns `StepResult` for final step.

---

## 10. Platform Layer

Trait definition:

```rust
pub trait Platform {
    fn build_command(&self, program: &str, args: &[String], ctx: &Context) -> Command;
    fn build_shell_command(&self, command: &str, ctx: &Context) -> Command;
    fn build_open_url(&self, url: &str, ctx: &Context) -> Command;
}
```

- Windows implementation: handles `cmd /C`, `start "" <url>`, working directory.
- Abstracts OS-specific quirks from executor.

---

## 11. Process Engine (`process.rs`)

- Pure execution logic.
- Responsibilities:
  - Spawn processes
  - Capture or stream stdout/stderr
  - Merge streams if needed
  - Regex matching in stream
  - Propagate pipeline values

- Fully OS-agnostic; executor and platform provide all input data.

---

## 13. Current Capabilities

- Sequential pipelines with `ctx.last` propagation.
- Shared context: args, vars, cwd, last.
- Transform engine: `Regex`, `Split`, `First`.
- Launch processes directly or via shell.
- Open URLs platform-independently.
- Pipeline-level error handling and early exit.
- Placeholder system fully supported in steps and transforms.
- Testable independently of UI and platform.

---

## 14. Known Design Debt / Future Directions (Ordered by Priority)

- **No Soft Error:** All type of error now stops the pipeline, No way to just warn and continue.
- **Argument ergonomics:** `Vec<String>` strict, YAML verbose → optional string parser, shell-first commands.

---

## 15. Summary

`velo_exec` is:

- **OS-agnostic:** all Windows-specific behavior is in the platform layer.
- **Pipeline-first:** sequential, placeholder-aware, transform-ready.
- **Testable independently:** pure Rust crate without Win32 or UI dependencies.
- **Composable:** supports complex pipelines, variable assignment, shell commands, and transforms.
- **Stable foundation for Velo’s future:** cross-platform pipeline engine decoupled from UI and OS logic.

---

## 16. Example Usage

```rust
use velo_exec::executor::system::DefaultSystem;
use velo_exec::platform::WindowsPlatform;
use velo_exec::*;

// Create an executor with system and platform
let exec = Executor::new(DefaultSystem, WindowsPlatform);

// Define a simple pipeline
let steps = vec![
    // Step 1: Capture shell output
    Step::Action {
        action: Action::Shell {
            command: "echo Hello Velo".into(),
            mode: ExecMode::Capture,
        },
        assign_to: Some("greeting".into()),
    },
    // Step 2: Extract a part using regex
    Step::Action {
        action: Action::Transform(Transform::Regex {
            input: Some("{var:greeting}".into()), // use previously assigned variable
            pattern: r"Hello (.+)".into(),
            group: 1,
        }),
        assign_to: Some("name".into()),
    },
    // Step 3: Open a URL (fire-and-forget)
    Step::Action {
        action: Action::OpenUrl {
            url: "https://example.com".into(),
        },
        assign_to: None,
    },
];

// Initialize context (no CLI args here)
let mut ctx = Context::new(vec![]);

// Run the pipeline
let result = exec.run(&steps, &mut ctx);

// Inspect results
println!("Pipeline success: {}", result.success);
println!("Last value: {:?}", ctx.last);
println!("Assigned variables: {:?}", ctx.vars);

// Expected output:
// Pipeline success: true
// Last value: Value::None (last step was OpenUrl)
// Assigned variables: {"greeting": "Hello Velo", "name": "Velo"}
```

**Notes:**

- `{var:<name>}` placeholders allow reusing variables in later steps.
- `ctx.last` is automatically updated after each step and can be used for chaining.
- `ExecMode::Capture` captures output, `FireForget` launches processes without waiting.

## 17. YAML Example

```YAML
commands:
  - name: Smart Search
    description: "Search, extract first match, and open it"
    aliases:
      - search
      - rg
      - find

    steps:
      # Step 1: run ripgrep with inline arg defaults
      - action:
          type: process
          program: "rg"
          args:
            - "{arg:query = 'fn'}"
            - "{arg:path = '.'}"
            - "--max-count"
            - "100"
          mode: capture
        assign_to: "output"

      # Step 2: extract first matching file
      - action:
          type: transform
          transform: regex
          input: "{var:output}"
          pattern: "^([^:\n]+)"
          group: 1
        assign_to: "file"

      # Step 3: demonstrate ctx.last chaining
      - action:
          type: transform
          transform: regex
          input: "{last}"
          pattern: "(.+)"
          group: 1
        assign_to: "clean_file"

      # Step 4: use arg + var in shell
      - action:
          type: shell
          command: "echo Searching {arg:query = 'fn'} && code {var:clean_file}"
          mode: fire_forget
```
