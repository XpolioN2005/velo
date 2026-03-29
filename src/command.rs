use serde::Deserialize;
use std::collections::HashMap;
use std::rc::Rc;
use velo_exec::Action;
use velo_exec::core::Step;

#[derive(Deserialize)]
pub struct Command {
    pub name: String,
    pub description: String,
    pub aliases: Vec<String>,
    pub steps: Vec<Step>,

    pub arg_order: Vec<String>,
    pub arg_defaults: HashMap<String, String>,
}

pub type CommandRegistry = Vec<Rc<Command>>;

pub fn build_command_lookup(commands: &CommandRegistry) -> HashMap<String, Rc<Command>> {
    let mut map = HashMap::new();
    for cmd in commands {
        map.insert(cmd.name.clone(), Rc::clone(cmd));
        for alias in &cmd.aliases {
            map.insert(alias.clone(), Rc::clone(cmd));
        }
    }
    map
}

pub fn built_in_commands() -> CommandRegistry {
    vec![
        Rc::new(Command {
            name: "Quit".into(),
            description: "Close the application".into(),
            aliases: vec!["exit".into(), "quit".into()],
            steps: vec![Step {
                action: Action::System {
                    action: "internal.close_app".into(),
                },
                assign_to: None,
            }],
            arg_order: vec![],
            arg_defaults: HashMap::new(),
        }),
        Rc::new(Command {
            name: "Reload".into(),
            description: "Reload config and commands".into(),
            aliases: vec!["refresh".into()],
            steps: vec![Step {
                action: Action::System {
                    action: "internal.reload_config".into(),
                },
                assign_to: None,
            }],
            arg_order: vec![],
            arg_defaults: HashMap::new(),
        }),
    ]
}

pub fn load_all_commands() -> CommandRegistry {
    let mut cmds = built_in_commands();
    let user_cmds = crate::config::load_user_commands();

    for user in user_cmds {
        if cmds.iter().any(|c| {
            c.name == user.name
                || c.aliases.contains(&user.name)
                || user
                    .aliases
                    .iter()
                    .any(|a| a == &c.name || c.aliases.contains(a))
        }) {
            println!("Skipping duplicate command: {}", user.name);
            continue;
        }
        cmds.push(user);
    }

    cmds
}
