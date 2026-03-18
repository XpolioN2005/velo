use crate::command::{Action, InternalAction, UserAction, WindowAction, substitute};

pub fn run_builtin(action: &Action, args: &[String], offset: usize) -> WindowAction {
    match action {
        Action::Internal(InternalAction::Quit) => WindowAction::Quit,
        Action::Internal(InternalAction::Hide) => WindowAction::Hide,
        Action::Internal(InternalAction::ReloadConfig) => WindowAction::Hide,
        Action::LaunchProcess {
            program,
            args: templates,
            prompts,
        } => {
            let final_args: Vec<String> = templates
                .iter()
                .map(|t| substitute(t, &args[offset..offset + prompts.len()]))
                .collect();
            let _ = std::process::Command::new(program)
                .args(&final_args)
                .spawn();
            WindowAction::Hide
        }
        Action::OpenUrl { url, prompts } => {
            let final_url = substitute(url, &args[offset..offset + prompts.len()]);
            open_url(&final_url);
            WindowAction::Hide
        }
        Action::Compound(steps) => {
            let mut off = offset;
            for step in steps {
                run_builtin(step, args, off);
                off += step.all_prompts().len();
            }
            WindowAction::Hide
        }
    }
}

pub fn run_user(action: &UserAction, args: &[String], offset: usize) -> WindowAction {
    match action {
        UserAction::LaunchProcess {
            program,
            args: templates,
            prompts,
        } => {
            let final_args: Vec<String> = templates
                .iter()
                .map(|t| substitute(t, &args[offset..offset + prompts.len()]))
                .collect();
            let _ = std::process::Command::new(program)
                .args(&final_args)
                .spawn();
            WindowAction::Hide
        }
        UserAction::OpenUrl { url, prompts } => {
            let final_url = substitute(url, &args[offset..offset + prompts.len()]);
            open_url(&final_url);
            WindowAction::Hide
        }
        UserAction::Compound(steps) => {
            let mut off = offset;
            for step in steps {
                run_user(step, args, off);
                off += step.all_prompts().len();
            }
            WindowAction::Hide
        }
    }
}

fn open_url(url: &str) {
    let _ = std::process::Command::new("cmd")
        .args(["/c", "start", "", url])
        .spawn();
}
