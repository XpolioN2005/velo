use crate::command::{OnFailure, UserAction, UserCommand, UserPrompt};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
# velo configuration\n\
\n\
# window position — set automatically on drag, or override manually\n\
# window_x: 960\n\
# window_y: 400\n\
";

    let _ = std::fs::write(path, default);
}

// ── commands.yaml ─────────────────────────────────────────────────────────────

mod yaml_types {
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
        pub action: RawAction,
    }

    #[derive(Deserialize)]
    pub struct RawPrompt {
        pub label: String,
        #[serde(default)]
        pub optional: bool,
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum RawOnFailure {
        Stop,
        Continue,
    }

    impl Default for RawOnFailure {
        fn default() -> Self {
            RawOnFailure::Stop
        }
    }

    #[derive(Deserialize)]
    #[serde(tag = "type", rename_all = "snake_case")]
    pub enum RawAction {
        Launch {
            program: String,
            args: Vec<String>,
            #[serde(default)]
            prompts: Vec<RawPrompt>,
        },
        OpenUrl {
            url: String,
            #[serde(default)]
            prompts: Vec<RawPrompt>,
        },
        Compound {
            steps: Vec<RawAction>,
            // mode: "sequential" | "parallel" — default parallel
            #[serde(default)]
            mode: RawCompoundMode,
            #[serde(default)]
            on_failure: RawOnFailure,
        },
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "snake_case")]
    pub enum RawCompoundMode {
        Parallel,
        Sequential,
    }

    impl Default for RawCompoundMode {
        fn default() -> Self {
            RawCompoundMode::Parallel
        }
    }
}

fn raw_prompt(p: yaml_types::RawPrompt) -> UserPrompt {
    UserPrompt {
        label: p.label,
        optional: p.optional,
    }
}

fn raw_on_failure(r: yaml_types::RawOnFailure) -> OnFailure {
    match r {
        yaml_types::RawOnFailure::Stop => OnFailure::Stop,
        yaml_types::RawOnFailure::Continue => OnFailure::Continue,
    }
}

fn raw_to_user_action(raw: yaml_types::RawAction) -> UserAction {
    match raw {
        yaml_types::RawAction::Launch {
            program,
            args,
            prompts,
        } => UserAction::LaunchProcess {
            program,
            args,
            prompts: prompts.into_iter().map(raw_prompt).collect(),
        },
        yaml_types::RawAction::OpenUrl { url, prompts } => UserAction::OpenUrl {
            url,
            prompts: prompts.into_iter().map(raw_prompt).collect(),
        },
        yaml_types::RawAction::Compound {
            steps,
            mode,
            on_failure,
        } => UserAction::Compound {
            steps: steps.into_iter().map(raw_to_user_action).collect(),
            sequential: matches!(mode, yaml_types::RawCompoundMode::Sequential),
            on_failure: raw_on_failure(on_failure),
        },
    }
}

pub fn load_user_commands() -> Vec<UserCommand> {
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
        .map(|raw| UserCommand {
            name: raw.name,
            description: raw.description,
            aliases: raw.aliases,
            action: raw_to_user_action(raw.action),
        })
        .collect()
}
