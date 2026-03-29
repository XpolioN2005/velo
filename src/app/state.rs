use super::MatchedCommand;
use super::search;
use super::system::AppSystem;
use crate::command::{Command, CommandRegistry, load_all_commands};
use crate::config::{AppConfig, load_config, save_config};
use crate::window::state::WindowAction;
use std::rc::Rc;
use velo_exec::executor::system::DefaultSystem;
use velo_exec::platform::WindowsPlatform;
use velo_exec::{Context, Executor, Value};

pub enum InputMode {
    Query,
    ArgInput {
        command: Rc<Command>,
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
    pub command_registry: CommandRegistry,
    pub results: Vec<MatchedCommand>,
    pub config: AppConfig,
    pub cursor: usize,
    pub selection: Option<(usize, usize)>,
    pub executor: Executor<AppSystem, WindowsPlatform>,
}

impl AppState {
    pub fn new() -> Self {
        let command_registry = load_all_commands();
        let config = load_config();
        let executor = Executor::new(
            AppSystem {
                default: DefaultSystem,
            },
            WindowsPlatform,
        );

        let mut state = Self {
            query: String::new(),
            arg_buffer: String::new(),
            mode: InputMode::Query,
            focused: true,
            selected: 0,
            command_registry,
            results: Vec::new(),
            config,
            cursor: 0,
            selection: None,
            executor,
        };
        state.rebuild_results();
        state
    }

    pub fn save_position(&mut self, x: i32, y: i32) {
        self.config.window_x = Some(x);
        self.config.window_y = Some(y);
        save_config(&self.config);
    }

    pub fn active_buf(&self) -> &str {
        match self.mode {
            InputMode::Query => &self.query,
            InputMode::ArgInput { .. } => &self.arg_buffer,
        }
    }

    fn active_buf_mut(&mut self) -> &mut String {
        match self.mode {
            InputMode::Query => &mut self.query,
            InputMode::ArgInput { .. } => &mut self.arg_buffer,
        }
    }

    fn active_limit(&self) -> usize {
        match self.mode {
            InputMode::Query => 100,
            InputMode::ArgInput { .. } => 200,
        }
    }

    // ── selection / cursor helpers ──────────────────────────────

    pub fn select_all(&mut self) {
        let len = self.active_buf().len();
        if len > 0 {
            self.selection = Some((0, len));
            self.cursor = len;
        }
    }

    fn delete_selection(&mut self) -> usize {
        let (s, e) = match self.selection.take() {
            Some(range) => range,
            None => return self.cursor,
        };
        self.active_buf_mut().drain(s..e);
        self.cursor = s;
        s
    }

    pub fn selected_text(&self) -> Option<&str> {
        let (s, e) = self.selection?;
        self.active_buf().get(s..e)
    }

    pub fn push_char(&mut self, c: char) {
        let start = if self.selection.is_some() {
            self.delete_selection()
        } else {
            self.cursor
        };

        let limit = self.active_limit();
        let buf = self.active_buf_mut();

        if buf.len() < limit {
            buf.insert(start, c);
            self.cursor = start + c.len_utf8();
        }

        if matches!(self.mode, InputMode::Query) {
            self.selected = 0;
            self.rebuild_results();
        }
    }

    pub fn pop_char(&mut self) {
        if self.selection.is_some() {
            self.delete_selection();
            if matches!(self.mode, InputMode::Query) {
                self.selected = 0;
                self.rebuild_results();
            }
            return;
        }

        if self.cursor == 0 {
            return;
        }

        let new_cursor = {
            let buf = self.active_buf();
            self.prev_char_boundary(buf, self.cursor)
        };

        let buf = self.active_buf_mut();
        buf.remove(new_cursor);
        self.cursor = new_cursor;

        if matches!(self.mode, InputMode::Query) {
            self.selected = 0;
            self.rebuild_results();
        }
    }

    pub fn move_cursor_left(&mut self) {
        let prev_cursor = self.prev_char_boundary(self.active_buf(), self.cursor);
        self.cursor = self
            .selection
            .take()
            .map(|(start, _)| start)
            .unwrap_or(prev_cursor);
    }

    pub fn move_cursor_right(&mut self) {
        let next_cursor = self.next_char_boundary(self.active_buf(), self.cursor);
        self.cursor = self
            .selection
            .take()
            .map(|(_, end)| end)
            .unwrap_or(next_cursor);
    }

    pub fn copy_text(&self) -> Option<String> {
        self.selected_text().map(|s| s.to_owned())
    }

    pub fn cut_text(&mut self) -> Option<String> {
        let text = self.selected_text()?.to_owned();
        self.delete_selection();
        if matches!(self.mode, InputMode::Query) {
            self.selected = 0;
            self.rebuild_results();
        }
        Some(text)
    }

    pub fn paste_text(&mut self, text: &str) {
        let available = {
            let buf = self.active_buf();
            self.active_limit().saturating_sub(buf.len())
        };

        let insert: String = text.chars().take(available).collect();

        if self.selection.is_some() {
            self.delete_selection();
        }

        let cursor = self.cursor;
        let buf = self.active_buf_mut();

        buf.insert_str(cursor, &insert);
        self.cursor += insert.len();

        if matches!(self.mode, InputMode::Query) {
            self.selected = 0;
            self.rebuild_results();
        }
    }

    fn prev_char_boundary(&self, s: &str, pos: usize) -> usize {
        let mut p = pos.saturating_sub(1);
        while p > 0 && !s.is_char_boundary(p) {
            p -= 1;
        }
        p
    }

    fn next_char_boundary(&self, s: &str, pos: usize) -> usize {
        let mut p = pos + 1;
        while p < s.len() && !s.is_char_boundary(p) {
            p += 1;
        }
        p
    }

    pub fn clear_query(&mut self) {
        self.query.clear();
        self.arg_buffer.clear();
        self.mode = InputMode::Query;
        self.selected = 0;
        self.cursor = 0;
        self.selection = None;
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
            } => command.arg_order.get(*prompt_index).map(|s| s.as_str()),
        }
    }

    pub fn enter(&mut self, hwnd: isize) -> WindowAction {
        match &self.mode {
            InputMode::Query => {
                if self.results.is_empty() {
                    return WindowAction::Nothing;
                }

                let cmd = self.results[self.selected].cmd.clone();
                if cmd.arg_order.is_empty() {
                    return self.execute_cmd(cmd, vec![], hwnd);
                }

                self.mode = InputMode::ArgInput {
                    command: cmd.clone(),
                    prompt_index: 0,
                    collected_args: Vec::new(),
                };

                self.arg_buffer = cmd
                    .arg_order
                    .get(0)
                    .and_then(|arg| cmd.arg_defaults.get(arg).cloned())
                    .unwrap_or_default();
                self.cursor = self.arg_buffer.len();
                self.selection = None;

                WindowAction::Nothing
            }
            InputMode::ArgInput { .. } => self.advance_arg(hwnd),
        }
    }

    pub fn escape(&mut self) {
        if let InputMode::ArgInput { .. } = self.mode {
            self.mode = InputMode::Query;
            self.arg_buffer.clear();
            self.cursor = self.query.len();
            self.selection = None;
        }
    }

    pub fn escape_should_hide(&self) -> bool {
        matches!(self.mode, InputMode::Query)
    }

    fn advance_arg(&mut self, hwnd: isize) -> WindowAction {
        let (command, optional, collected_args, prompt_index) = match &mut self.mode {
            InputMode::ArgInput {
                command,
                prompt_index,
                collected_args,
            } => {
                let optional = command
                    .arg_order
                    .get(*prompt_index)
                    .map(|arg| command.arg_defaults.contains_key(arg))
                    .unwrap_or(true);
                (command.clone(), optional, collected_args, prompt_index)
            }
            _ => return WindowAction::Nothing,
        };

        if self.arg_buffer.is_empty() && !optional {
            return WindowAction::Nothing;
        }

        let arg = std::mem::take(&mut self.arg_buffer);
        collected_args.push(arg);
        *prompt_index += 1;

        if *prompt_index >= command.arg_order.len() {
            let args = collected_args.clone();
            self.mode = InputMode::Query;
            return self.execute_cmd(command, args, hwnd);
        }

        self.arg_buffer = command
            .arg_order
            .get(*prompt_index)
            .and_then(|arg| command.arg_defaults.get(arg).cloned())
            .unwrap_or_default();
        self.cursor = self.arg_buffer.len();
        self.selection = None;

        WindowAction::Nothing
    }

    fn execute_cmd(&self, cmd: Rc<Command>, args: Vec<String>, _hwnd: isize) -> WindowAction {
        let mut ctx = Context::new(args);
        let result = self.executor.run(&cmd.steps, &mut ctx);

        if !result.success {
            if let Some(err) = &result.error {
                eprintln!("Command error: {}", err); // log the error
            }
            return WindowAction::Nothing;
        }

        match &result.value {
            Value::String(s) if s == "quit" => WindowAction::Quit,
            _ => WindowAction::Hide,
        }
    }

    fn rebuild_results(&mut self) {
        self.results = search::build_results(&self.query, &self.command_registry);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
