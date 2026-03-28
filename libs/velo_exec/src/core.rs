use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

// ── values ───────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    None,
    String(String),
    Bool(bool),
    Number(f64),
    List(Vec<Value>),
}

// ── context ──────────────────────────────────────────────────────────────────

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

// ── system ───────────────────────────────────────────────────────────────────

pub struct StepResult {
    pub success: bool,
    pub value: Value,
    pub error: Option<String>,
}

pub trait SystemHandler {
    fn run(&self, action: &str, ctx: &mut Context) -> StepResult;
}

// ── execution mode ───────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecMode {
    FireForget,
    Capture,
    Stream,
    StreamMatch { pattern: String },
}

impl ExecMode {
    pub fn compile(self) -> Self {
        match self {
            ExecMode::StreamMatch { pattern } => {
                // Validate regex early
                let _ = Regex::new(&pattern).expect("Invalid regex pattern");
                ExecMode::StreamMatch { pattern }
            }
            other => other,
        }
    }
}

// ── transforms ───────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "transform", rename_all = "snake_case")]
pub enum Transform {
    Regex {
        input: Option<String>,
        pattern: String,
        group: usize,
    },
    Split {
        input: Option<String>,
        delimiter: String,
    },
    First {
        input: Option<String>,
    },
}

// ── actions ──────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    Process {
        program: String,
        args: Vec<String>,
        mode: ExecMode,
    },
    Shell {
        command: String,
        mode: ExecMode,
    },
    OpenUrl {
        url: String,
    },
    System {
        action: String,
    },
    Transform(Transform),
}

// ── step ─────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Step {
    pub action: Action,

    #[serde(default)]
    pub assign_to: Option<String>,
}
