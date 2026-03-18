use super::MatchedCommand;
use super::{executor, search};
use crate::command::{BUILT_INS, CommandRef, UserCommand, WindowAction};
use crate::config::{AppConfig, load_config, load_user_commands, save_config};

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
    pub results: Vec<MatchedCommand>,
    pub config: AppConfig,
}

impl AppState {
    pub fn new() -> Self {
        let user_commands = load_user_commands();
        let config = load_config();
        let mut state = Self {
            query: String::new(),
            arg_buffer: String::new(),
            mode: InputMode::Query,
            focused: true,
            selected: 0,
            user_commands,
            results: Vec::new(),
            config,
        };
        state.rebuild_results();
        state
    }

    pub fn save_position(&mut self, x: i32, y: i32) {
        self.config.window_x = Some(x);
        self.config.window_y = Some(y);
        save_config(&self.config);
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
                ..
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
            CommandRef::BuiltIn(idx) => executor::run_builtin(&BUILT_INS[idx].action, &args, 0),
            CommandRef::User(idx) => executor::run_user(&self.user_commands[idx].action, &args, 0),
        }
    }

    fn rebuild_results(&mut self) {
        self.results = search::build_results(&self.query, &self.user_commands);
    }
}
