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

pub struct MatchedCommand {
    pub cmd_ref: CommandRef,
    pub match_indices: Vec<usize>,
    pub score: i32,
}

pub struct AppState {
    pub query: String,
    pub arg_buffer: String,
    pub mode: InputMode,
    pub focused: bool,
    pub selected: usize,
    pub user_commands: Vec<UserCommand>,
    pub results: Vec<MatchedCommand>,
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
                let cmd_ref = self.results[self.selected].cmd_ref;
                let prompts = self.get_prompts(cmd_ref);
                if prompts.is_empty() {
                    self.execute_cmd(cmd_ref, vec![])
                } else {
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
            InputMode::Query => {}
        }
    }

    pub fn escape_should_hide(&self) -> bool {
        matches!(self.mode, InputMode::Query)
    }

    fn advance_arg(&mut self) -> WindowAction {
        let (command, optional) = match &self.mode {
            InputMode::ArgInput {
                command,
                prompt_index,
                collected_args: _,
            } => {
                let prompts = self.get_prompts(*command);
                let optional = prompts.get(*prompt_index).map(|p| p.1).unwrap_or(true);
                (*command, optional)
            }
            _ => return WindowAction::Nothing,
        };

        if self.arg_buffer.is_empty() && !optional {
            return WindowAction::Nothing;
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
            if let Some((score, indices)) = fuzzy_match(cmd.name, &q) {
                self.results.push(MatchedCommand {
                    cmd_ref: CommandRef::BuiltIn(i),
                    match_indices: indices,
                    score,
                });
            } else {
                let alias_score = cmd
                    .aliases
                    .iter()
                    .filter_map(|a| fuzzy_match(a, &q))
                    .map(|(s, _)| s + 15)
                    .max();
                if let Some(score) = alias_score {
                    self.results.push(MatchedCommand {
                        cmd_ref: CommandRef::BuiltIn(i),
                        match_indices: vec![],
                        score,
                    });
                }
            }
        }

        for (i, cmd) in self.user_commands.iter().enumerate() {
            if let Some((score, indices)) = fuzzy_match(&cmd.name, &q) {
                self.results.push(MatchedCommand {
                    cmd_ref: CommandRef::User(i),
                    match_indices: indices,
                    score,
                });
            } else {
                let alias_score = cmd
                    .aliases
                    .iter()
                    .filter_map(|a| fuzzy_match(a, &q))
                    .map(|(s, _)| s + 15)
                    .max();
                if let Some(score) = alias_score {
                    self.results.push(MatchedCommand {
                        cmd_ref: CommandRef::User(i),
                        match_indices: vec![],
                        score,
                    });
                }
            }
        }

        self.results.sort_by(|a, b| b.score.cmp(&a.score));
    }
}

fn fuzzy_match(name: &str, query: &str) -> Option<(i32, Vec<usize>)> {
    let name_lower = name.to_lowercase();
    let name_chars: Vec<char> = name_lower.chars().collect();
    let query_chars: Vec<char> = query.chars().collect();

    let mut indices = Vec::new();
    let mut score = 0i32;
    let mut ni = 0;
    let mut qi = 0;
    let mut last_match = None::<usize>;
    let mut consecutive = 0i32;

    while qi < query_chars.len() && ni < name_chars.len() {
        if name_chars[ni] == query_chars[qi] {
            indices.push(ni);

            if ni > 0 && last_match == Some(ni - 1) {
                consecutive += 1;
                score += 5 * consecutive;
            } else {
                consecutive = 0;
            }

            if ni == 0 {
                score += 10;
            }

            if ni > 0 && (name_chars[ni - 1] == ' ' || name_chars[ni - 1] == '_') {
                score += 8;
            }

            score += 1;
            last_match = Some(ni);
            qi += 1;
        }
        ni += 1;
    }

    if qi == query_chars.len() {
        Some((score, indices))
    } else {
        None
    }
}

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
