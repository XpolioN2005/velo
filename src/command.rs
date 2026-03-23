#[derive(Clone)]
pub enum InternalAction {
    Quit,
    Hide,
    ReloadConfig,
}

#[derive(Clone)]
pub struct Prompt {
    pub label: &'static str,
    pub optional: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum OnFailure {
    Stop,
    Continue,
}

// Output of a completed sequential step — available as {stdout} / {exit_code}
#[derive(Clone)]
pub struct StepOutput {
    pub stdout: String,
    pub exit_code: i32,
}

#[derive(Clone)]
pub enum Action {
    Internal(InternalAction),
    LaunchProcess {
        program: &'static str,
        args: &'static [&'static str],
        prompts: &'static [Prompt],
    },
    OpenUrl {
        url: &'static str,
        prompts: &'static [Prompt],
    },
    Compound {
        steps: Vec<Action>,
        sequential: bool,
        on_failure: OnFailure,
    },
}

impl Action {
    pub fn all_prompts(&self) -> Vec<&Prompt> {
        match self {
            Action::LaunchProcess { prompts, .. } => prompts.iter().collect(),
            Action::OpenUrl { prompts, .. } => prompts.iter().collect(),
            Action::Compound { steps, .. } => steps.iter().flat_map(|s| s.all_prompts()).collect(),
            Action::Internal(_) => vec![],
        }
    }
}

pub struct BuiltInCommand {
    pub name: &'static str,
    pub description: &'static str,
    pub aliases: &'static [&'static str],
    pub action: Action,
}

pub struct UserCommand {
    pub name: String,
    pub description: String,
    pub aliases: Vec<String>,
    pub action: UserAction,
}

#[derive(Clone)]
pub struct UserPrompt {
    pub label: String,
    pub optional: bool,
}

#[derive(Clone)]
pub enum UserAction {
    LaunchProcess {
        program: String,
        args: Vec<String>,
        prompts: Vec<UserPrompt>,
    },
    OpenUrl {
        url: String,
        prompts: Vec<UserPrompt>,
    },
    Compound {
        steps: Vec<UserAction>,
        sequential: bool,
        on_failure: OnFailure,
    },
}

impl UserAction {
    pub fn all_prompts(&self) -> Vec<&UserPrompt> {
        match self {
            UserAction::LaunchProcess { prompts, .. } => prompts.iter().collect(),
            UserAction::OpenUrl { prompts, .. } => prompts.iter().collect(),
            UserAction::Compound { steps, .. } => {
                steps.iter().flat_map(|s| s.all_prompts()).collect()
            }
        }
    }
}

#[derive(Clone, Copy)]
pub enum CommandRef {
    BuiltIn(usize),
    User(usize),
}

#[derive(Clone, Copy, PartialEq)]
pub enum WindowAction {
    Quit,
    Hide,
    Nothing,
    RunSequence,
}

// Substitute {0},{1} positional args and optionally {stdout}/{exit_code}
// from the previous sequential step.
pub fn substitute(template: &str, args: &[String], prev: Option<&StepOutput>) -> String {
    let mut result = template.to_string();

    // positional args
    for (i, arg) in args.iter().enumerate() {
        result = result.replace(&format!("{{{}}}", i), arg);
    }

    // previous step output
    if let Some(p) = prev {
        result = result.replace("{stdout}", &p.stdout.trim().to_string());
        result = result.replace("{exit_code}", &p.exit_code.to_string());
    }

    result
}

pub static BUILT_INS: &[BuiltInCommand] = &[
    BuiltInCommand {
        name: "Quit Velo",
        description: "Exit the application",
        aliases: &["exit", "close"],
        action: Action::Internal(InternalAction::Quit),
    },
    BuiltInCommand {
        name: "Reload Config",
        description: "Reload commands.yaml without restarting",
        aliases: &["refresh"],
        action: Action::Internal(InternalAction::ReloadConfig),
    },
    BuiltInCommand {
        name: "Open PowerShell",
        description: "Launch PowerShell terminal",
        aliases: &["powershell", "ps", "terminal"],
        action: Action::LaunchProcess {
            program: "powershell.exe",
            args: &[],
            prompts: &[],
        },
    },
    BuiltInCommand {
        name: "Ping",
        description: "Ping a host",
        aliases: &["ping"],
        action: Action::LaunchProcess {
            program: "powershell.exe",
            args: &["-NoExit", "-Command", "ping {0}"],
            prompts: &[Prompt {
                label: "Host:",
                optional: false,
            }],
        },
    },
    BuiltInCommand {
        name: "Google Search",
        description: "Search Google with optional site filter",
        aliases: &["google", "g"],
        action: Action::OpenUrl {
            url: "https://www.google.com/search?q={0}+site:{1}",
            prompts: &[
                Prompt {
                    label: "Search query:",
                    optional: false,
                },
                Prompt {
                    label: "Site (optional):",
                    optional: true,
                },
            ],
        },
    },
];
