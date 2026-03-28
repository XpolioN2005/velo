use crate::command::{Command, CommandRegistry};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::rc::Rc;
use velo_exec::core::{Action, Step, Transform};

// ── paths ────────────────────────────────────────────────────────────────────

fn velo_dir() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|base| PathBuf::from(base).join("velo"))
}

fn commands_path() -> Option<PathBuf> {
    velo_dir().map(|d| d.join("commands.yaml"))
}

fn config_path() -> Option<PathBuf> {
    velo_dir().map(|d| d.join("config.yaml"))
}

// ── AppConfig ─────────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_x: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_y: Option<i32>,
}

pub fn load_config() -> AppConfig {
    let path = match config_path() {
        Some(p) => p,
        None => return AppConfig::default(),
    };

    if !path.exists() {
        ensure_default_config(&path);
        return AppConfig::default();
    }

    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return AppConfig::default(),
    };

    serde_yaml::from_str(&text).unwrap_or_default()
}

pub fn save_config(cfg: &AppConfig) {
    let path = match config_path() {
        Some(p) => p,
        None => return,
    };

    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }

    if let Ok(text) = serde_yaml::to_string(cfg) {
        let _ = std::fs::write(&path, text);
    }
}

fn ensure_default_config(path: &PathBuf) {
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }

    let default = "\
# velo configuration

# window position — set automatically on drag, or override manually
# window_x: 960
# window_y: 400
";

    let _ = std::fs::write(path, default);
}

// ── commands.yaml ─────────────────────────────────────────────────────────────

mod yaml_types {
    use super::*;
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct ConfigFile {
        #[serde(default)]
        pub commands: Vec<RawCommand>,
    }

    #[derive(Deserialize)]
    pub struct RawCommand {
        pub name: String,
        pub description: String,
        #[serde(default)]
        pub aliases: Vec<String>,
        pub steps: Vec<Step>,
    }
}

// ── ARG PARSER ────────────────────────────────────────────────────────────────

#[derive(Default)]
struct ArgSpec {
    order: Vec<String>,
    defaults: HashMap<String, String>,
}

fn process_steps(steps: &mut [Step]) -> ArgSpec {
    let mut spec = ArgSpec::default();
    for step in steps {
        process_step(step, &mut spec);
    }
    spec
}

fn process_step(step: &mut Step, spec: &mut ArgSpec) {
    match &mut step.action {
        Action::Process { program, args, .. } => {
            rewrite_string(program, spec);
            for arg in args {
                rewrite_string(arg, spec);
            }
        }

        Action::Shell { command, .. } => {
            rewrite_string(command, spec);
        }

        Action::OpenUrl { url } => {
            rewrite_string(url, spec);
        }

        Action::Transform(t) => match t {
            Transform::Regex { input, pattern, .. } => {
                // validate regex early
                let _ = Regex::new(pattern).expect("Invalid regex in transform");

                if let Some(s) = input {
                    rewrite_string(s, spec);
                }
            }
            Transform::Split { input, .. } => {
                if let Some(s) = input {
                    rewrite_string(s, spec);
                }
            }
            Transform::First { input } => {
                if let Some(s) = input {
                    rewrite_string(s, spec);
                }
            }
        },

        Action::System { .. } => {}
    }
}

// core rewrite logic
fn rewrite_string(input: &mut String, spec: &mut ArgSpec) {
    let re = Regex::new(r"\{arg:\s*([a-zA-Z0-9_]+)(?:\s*=\s*'([^']*)')?\}").unwrap();

    let mut result = input.clone();

    for cap in re.captures_iter(input) {
        let name = cap[1].to_string();
        let default = cap.get(2).map(|m| m.as_str().to_string());

        let index = if let Some(pos) = spec.order.iter().position(|n| n == &name) {
            pos
        } else {
            let pos = spec.order.len();
            spec.order.push(name.clone());

            if let Some(def) = default {
                spec.defaults.insert(name.clone(), def);
            }

            pos
        };

        result = result.replace(&cap[0], &format!("{{{}}}", index));
    }

    *input = result;
}

// ── LOAD COMMANDS ─────────────────────────────────────────────────────────────

pub fn load_user_commands() -> CommandRegistry {
    let path = match commands_path() {
        Some(p) => p,
        None => return vec![],
    };

    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return vec![],
    };

    let parsed: yaml_types::ConfigFile = match serde_yaml::from_str(&text) {
        Ok(p) => p,
        Err(_) => return vec![],
    };

    parsed
        .commands
        .into_iter()
        .map(|mut raw| {
            let arg_spec = process_steps(&mut raw.steps);

            Rc::new(Command {
                name: raw.name,
                description: raw.description,
                aliases: raw.aliases,
                steps: raw.steps,

                // IMPORTANT: preserve arg metadata
                arg_order: arg_spec.order,
                arg_defaults: arg_spec.defaults,
            })
        })
        .collect()
}
