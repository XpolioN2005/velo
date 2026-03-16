#[derive(Clone, Copy)]
pub enum Category {
    General,
    Shell,
    Settings,
    Navigation,
}

#[derive(Clone, Copy)]
pub enum InternalAction {
    Quit,
    ReloadConfig,
}

#[derive(Clone, Copy)]
pub enum BuiltInAction {
    Internal(InternalAction),
    // LaunchProcess and OpenUrl only make sense for user commands
}

pub struct BuiltInCommand {
    pub name: &'static str,
    pub description: &'static str,
    pub aliases: &'static [&'static str],
    pub category: Category,
    pub action: BuiltInAction,
}

pub enum UserAction {
    LaunchProcess(&'static str),
    OpenUrl(String),
}

pub struct UserCommand {
    pub name: String,
    pub description: String,
    pub aliases: Vec<String>,
    pub category: Category,
    pub action: UserAction,
}

// Lightweight reference into one of the two command lists
#[derive(Clone, Copy)]
pub enum CommandRef {
    BuiltIn(usize),
    User(usize),
}

pub static BUILT_INS: &[BuiltInCommand] = &[
    BuiltInCommand {
        name: "Quit Velo",
        description: "Exit the application",
        aliases: &["exit", "close"],
        category: Category::General,
        action: BuiltInAction::Internal(InternalAction::Quit),
    },
    BuiltInCommand {
        name: "Reload Config",
        description: "Reload commands.toml without restarting",
        aliases: &["refresh"],
        category: Category::General,
        action: BuiltInAction::Internal(InternalAction::ReloadConfig),
    },
    BuiltInCommand {
        name: "Hello Velo",
        description: "Test command — confirms display is working",
        aliases: &["test", "hello"],
        category: Category::General,
        action: BuiltInAction::Internal(InternalAction::Quit), // placeholder action
    },
];

// What window.rs should do after a command runs
#[derive(Clone)]
pub enum ExecuteResult {
    Quit,
    Hide,
    ReloadConfig,
    Nothing,
}
