use crate::command::{Category, UserAction, UserCommand};
use std::path::PathBuf;

// %APPDATA%\velo\commands.toml
fn config_path() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|base| PathBuf::from(base).join("velo").join("commands.toml"))
}

// TOML shape:
// [[commands]]
// name = "Open Docs"
// description = "Opens the Velo docs"
// aliases = ["docs"]
// action = { type = "OpenUrl", url = "https://..." }
mod toml_types {
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
    #[serde(tag = "type")]
    pub enum RawAction {
        LaunchProcess { cmd: String },
        OpenUrl { url: String },
    }
}

pub fn load_user_commands() -> Vec<UserCommand> {
    let path = match config_path() {
        Some(p) => p,
        None => return vec![],
    };

    let text = match std::fs::read_to_string(&path) {
        Ok(t) => t,
        Err(_) => return vec![], // missing file is fine
    };

    let parsed: toml_types::ConfigFile = match toml::from_str(&text) {
        Ok(p) => p,
        Err(_) => return vec![], // malformed config — silent fail for now
    };

    parsed
        .commands
        .into_iter()
        .map(|raw| {
            let action = match raw.action {
                toml_types::RawAction::LaunchProcess { cmd } => {
                    // leak is acceptable — small, lives for program lifetime
                    UserAction::LaunchProcess(Box::leak(cmd.into_boxed_str()))
                }
                toml_types::RawAction::OpenUrl { url } => UserAction::OpenUrl(url),
            };
            UserCommand {
                name: raw.name,
                description: raw.description,
                aliases: raw.aliases,
                category: Category::General,
                action,
            }
        })
        .collect()
}
