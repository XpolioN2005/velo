use crate::command::{
    Action, BUILT_INS, CommandRef, InternalAction, UserAction, UserCommand, WindowAction,
    substitute,
};
use crate::config::load_user_commands;

pub enum InputMode {
    Query,
    ArgInput {
        command: CommandRef,
        prompt_index: usize,
        collected_args: Vec<String>,
    },
}

pub struct AppState {
    pub query: String,
    pub arg_buffer: String,
    pub mode: InputMode,
    pub focused: bool,
    pub selected: usize,
    pub user_commands: Vec<UserCommand>,
    pub results: Vec<CommandRef>,
}

impl AppState {
    pub fn new() -> Self {
        let user_commands = load_user_commands();
        let mut state = Self {
            query: String::new(),
            arg_buffer: String::new(),
            mode: InputMode::Query,
            focused: true,
            selected: 0,
            user_commands,
            results: Vec::new(),
        };
        state.rebuild_results();
        state
    }

    pub fn push_char(&mut self, c: char) {
        match self.mode {
            InputMode::Query => {
                if self.query.len() < 100 {
                    self.query.push(c);
                    self.selected = 0;
                    self.rebuild_results();
                }
            }
            InputMode::ArgInput { .. } => {
                if self.arg_buffer.len() < 200 {
                    self.arg_buffer.push(c);
                }
            }
        }
    }

    pub fn pop_char(&mut self) {
        match self.mode {
            InputMode::Query => {
                self.query.pop();
                self.selected = 0;
                self.rebuild_results();
            }
            InputMode::ArgInput { .. } => {
                if self.arg_buffer.is_empty() {
                    // backspace on empty buffer — cancel back to query mode
                    self.mode = InputMode::Query;
                    self.arg_buffer.clear();
                } else {
                    self.arg_buffer.pop();
                }
            }
        }
    }

    pub fn clear_query(&mut self) {
        self.query.clear();
        self.arg_buffer.clear();
        self.mode = InputMode::Query;
        self.selected = 0;
        self.rebuild_results();
    }

    pub fn select_next(&mut self) {
        if matches!(self.mode, InputMode::Query) && !self.results.is_empty() {
            self.selected = (self.selected + 1).min(self.results.len() - 1);
        }
    }

    pub fn select_prev(&mut self) {
        if matches!(self.mode, InputMode::Query) {
            self.selected = self.selected.saturating_sub(1);
        }
    }

    // Current prompt label for palette rendering
    pub fn current_prompt(&self) -> Option<&str> {
        match &self.mode {
            InputMode::Query => None,
            InputMode::ArgInput {
                command,
                prompt_index,
                ..
            } => {
                let prompts = self.get_prompts(*command);
                prompts.get(*prompt_index).map(|p| p.0)
            }
        }
    }

    pub fn enter(&mut self) -> WindowAction {
        match &self.mode {
            InputMode::Query => {
                if self.results.is_empty() {
                    return WindowAction::Nothing;
                }
                let cmd_ref = self.results[self.selected];
                let prompts = self.get_prompts(cmd_ref);
                if prompts.is_empty() {
                    // no prompts — execute immediately
                    self.execute_cmd(cmd_ref, vec![])
                } else {
                    // has prompts — enter arg input mode
                    self.mode = InputMode::ArgInput {
                        command: cmd_ref,
                        prompt_index: 0,
                        collected_args: Vec::new(),
                    };
                    self.arg_buffer.clear();
                    WindowAction::Nothing
                }
            }
            InputMode::ArgInput { .. } => self.advance_arg(),
        }
    }

    pub fn escape(&mut self) {
        match self.mode {
            InputMode::ArgInput { .. } => {
                self.mode = InputMode::Query;
                self.arg_buffer.clear();
            }
            InputMode::Query => {} // window.rs handles hiding
        }
    }

    // Returns true if escape should hide the window (only in Query mode)
    pub fn escape_should_hide(&self) -> bool {
        matches!(self.mode, InputMode::Query)
    }

    fn advance_arg(&mut self) -> WindowAction {
        let (command, prompt_index, optional) = match &self.mode {
            InputMode::ArgInput {
                command,
                prompt_index,
                collected_args: _,
            } => {
                let prompts = self.get_prompts(*command);
                let optional = prompts.get(*prompt_index).map(|p| p.1).unwrap_or(true);
                (*command, *prompt_index, optional)
            }
            _ => return WindowAction::Nothing,
        };

        if self.arg_buffer.is_empty() && !optional {
            return WindowAction::Nothing; // reject empty non-optional
        }

        let arg = self.arg_buffer.clone();
        self.arg_buffer.clear();

        let prompts_len = self.get_prompts(command).len();

        if let InputMode::ArgInput {
            collected_args,
            prompt_index,
            ..
        } = &mut self.mode
        {
            collected_args.push(arg);
            *prompt_index += 1;

            if *prompt_index >= prompts_len {
                // all prompts filled — execute
                let args = collected_args.clone();
                self.mode = InputMode::Query;
                return self.execute_cmd(command, args);
            }
        }

        WindowAction::Nothing
    }

    fn get_prompts(&self, cmd_ref: CommandRef) -> Vec<(&str, bool)> {
        match cmd_ref {
            CommandRef::BuiltIn(idx) => BUILT_INS[idx]
                .action
                .all_prompts()
                .iter()
                .map(|p| (p.label, p.optional))
                .collect(),
            CommandRef::User(idx) => self.user_commands[idx]
                .action
                .all_prompts()
                .iter()
                .map(|p| (p.label.as_str(), p.optional))
                .collect(),
        }
    }

    fn execute_cmd(&self, cmd_ref: CommandRef, args: Vec<String>) -> WindowAction {
        match cmd_ref {
            CommandRef::BuiltIn(idx) => run_builtin(&BUILT_INS[idx].action, &args, 0),
            CommandRef::User(idx) => run_user(&self.user_commands[idx].action, &args, 0),
        }
    }

    fn rebuild_results(&mut self) {
        self.results.clear();
        if self.query.is_empty() {
            return;
        }
        let q = self.query.to_lowercase();
        for (i, cmd) in BUILT_INS.iter().enumerate() {
            if matches_query(cmd.name, cmd.aliases, &q) {
                self.results.push(CommandRef::BuiltIn(i));
            }
        }
        for (i, cmd) in self.user_commands.iter().enumerate() {
            let aliases: Vec<&str> = cmd.aliases.iter().map(|s| s.as_str()).collect();
            if matches_query(&cmd.name, &aliases, &q) {
                self.results.push(CommandRef::User(i));
            }
        }
    }
}

// offset = starting index into collected_args for this action's prompts
fn run_builtin(action: &Action, args: &[String], offset: usize) -> WindowAction {
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

fn run_user(action: &UserAction, args: &[String], offset: usize) -> WindowAction {
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

fn matches_query(name: &str, aliases: &[&str], q: &str) -> bool {
    name.to_lowercase().contains(q) || aliases.iter().any(|a| a.to_lowercase().contains(q))
}
