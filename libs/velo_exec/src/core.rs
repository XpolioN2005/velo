use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum Value {
    None,
    String(String),
    Bool(bool),
    Number(f64),
}

pub struct Context {
    pub vars: HashMap<String, Value>,
    pub args: Vec<String>,
    pub cwd: Option<PathBuf>,
    pub last: Value,
}

impl Context {
    pub fn new(args: Vec<String>) -> Self {
        Self {
            vars: HashMap::new(),
            args,
            cwd: None,
            last: Value::None,
        }
    }
}

// ── system ─────────────────────────────

#[derive(Copy, Clone, Debug)]
pub enum SystemActionId {
    GetCwd,
    SetCwd,
    JoinPath,
}

pub struct StepResult {
    pub success: bool,
    pub value: Value,
    pub error: Option<String>,
}

pub trait SystemHandler {
    fn run(&self, action: SystemActionId, ctx: &mut Context) -> StepResult;
}

// ── actions ───────────────────────────

pub enum ExecMode {
    FireForget,
    Capture,
    Stream,
    StreamMatch(regex::Regex),
}

pub enum Transform {
    Regex {
        input: Option<String>,
        pattern: String,
        group: usize,
    },
}

pub enum Action {
    LaunchProcess {
        program: String,
        args: Vec<String>,
        mode: ExecMode,
    },
    System(SystemActionId),
    Transform(Transform),
}

// ── steps ─────────────────────────────

pub enum Step {
    Action {
        action: Action,
        assign_to: Option<String>,
    },
}
