use crate::command::{UserAction, UserCommand, UserPrompt};
use std::path::PathBuf;

fn config_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|base| PathBuf::from(base).join("velo").join("commands.yaml"))
}

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
        },
    }
}

fn raw_prompt(p: yaml_types::RawPrompt) -> UserPrompt {
    UserPrompt {
        label: p.label,
        optional: p.optional,
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
        yaml_types::RawAction::Compound { steps } => {
            UserAction::Compound(steps.into_iter().map(raw_to_user_action).collect())
        }
    }
}

pub fn load_user_commands() -> Vec<UserCommand> {
    let path = match config_path() {
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
