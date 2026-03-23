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
    pub cursor: usize,
    pub selection: Option<(usize, usize)>,
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
            cursor: 0,
            selection: None,
        };
        state.rebuild_results();
        state
    }

    pub fn save_position(&mut self, x: i32, y: i32) {
        self.config.window_x = Some(x);
        self.config.window_y = Some(y);
        save_config(&self.config);
    }

    // ── active buffer helpers ─────────────────────────────────────────────────

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

    // ── selection helpers ─────────────────────────────────────────────────────

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
        let buf = self.active_buf_mut();
        buf.drain(s..e);
        self.cursor = s;
        s
    }

    pub fn selected_text(&self) -> Option<&str> {
        let (s, e) = self.selection?;
        self.active_buf().get(s..e)
    }

    // ── push_char ─────────────────────────────────────────────────────────────

    pub fn push_char(&mut self, c: char) {
        if self.selection.is_some() {
            let start = self.delete_selection();
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
            return;
        }

        let limit = self.active_limit();
        let cursor = self.cursor;
        let buf = self.active_buf_mut();
        if buf.len() < limit {
            buf.insert(cursor, c);
            self.cursor = cursor + c.len_utf8();
        }

        if matches!(self.mode, InputMode::Query) {
            self.selected = 0;
            self.rebuild_results();
        }
    }

    // ── pop_char ──────────────────────────────────────────────────────────────

    pub fn pop_char(&mut self) {
        if self.selection.is_some() {
            self.delete_selection();
            if matches!(self.mode, InputMode::Query) {
                self.selected = 0;
                self.rebuild_results();
            }
            return;
        }

        match self.mode {
            InputMode::Query => {
                if self.cursor > 0 {
                    let new_cursor = self.prev_char_boundary(&self.query.clone(), self.cursor);
                    self.query.remove(new_cursor);
                    self.cursor = new_cursor;
                    self.selected = 0;
                    self.rebuild_results();
                }
            }
            InputMode::ArgInput { .. } => {
                if self.arg_buffer.is_empty() {
                    self.mode = InputMode::Query;
                    self.arg_buffer.clear();
                    self.cursor = self.query.len();
                } else if self.cursor > 0 {
                    let new_cursor = self.prev_char_boundary(&self.arg_buffer.clone(), self.cursor);
                    self.arg_buffer.remove(new_cursor);
                    self.cursor = new_cursor;
                }
            }
        }
    }

    // ── arrow keys ────────────────────────────────────────────────────────────

    pub fn move_cursor_left(&mut self) {
        if let Some((start, _)) = self.selection.take() {
            self.cursor = start;
        } else {
            self.cursor = self.prev_char_boundary(self.active_buf(), self.cursor);
        }
    }

    pub fn move_cursor_right(&mut self) {
        if let Some((_, end)) = self.selection.take() {
            self.cursor = end;
        } else {
            self.cursor = self.next_char_boundary(self.active_buf(), self.cursor);
        }
    }

    // ── clipboard ─────────────────────────────────────────────────────────────

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
        if self.selection.is_some() {
            self.delete_selection();
        }
        let cursor = self.cursor;
        let limit = self.active_limit();
        let buf = self.active_buf_mut();
        let available = limit.saturating_sub(buf.len());
        let insert: String = text.chars().take(available).collect();
        let byte_len = insert.len();
        buf.insert_str(cursor, &insert);
        self.cursor = cursor + byte_len;
        if matches!(self.mode, InputMode::Query) {
            self.selected = 0;
            self.rebuild_results();
        }
    }

    // ── char boundary utils ───────────────────────────────────────────────────

    fn prev_char_boundary(&self, s: &str, pos: usize) -> usize {
        if pos == 0 {
            return 0;
        }
        let mut p = pos - 1;
        while p > 0 && !s.is_char_boundary(p) {
            p -= 1;
        }
        p
    }

    fn next_char_boundary(&self, s: &str, pos: usize) -> usize {
        if pos >= s.len() {
            return s.len();
        }
        let mut p = pos + 1;
        while p < s.len() && !s.is_char_boundary(p) {
            p += 1;
        }
        p
    }

    // ── clear_query ───────────────────────────────────────────────────────────

    pub fn clear_query(&mut self) {
        self.query.clear();
        self.arg_buffer.clear();
        self.mode = InputMode::Query;
        self.selected = 0;
        self.cursor = 0;
        self.selection = None;
        self.rebuild_results();
    }

    // ── result navigation ─────────────────────────────────────────────────────

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

    // ── prompt / enter / escape ───────────────────────────────────────────────

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

    // hwnd passed in so execute_cmd can hand it to the sequential worker
    pub fn enter(&mut self, hwnd: isize) -> WindowAction {
        match &self.mode {
            InputMode::Query => {
                if self.results.is_empty() {
                    return WindowAction::Nothing;
                }
                let cmd_ref = self.results[self.selected].cmd_ref;
                let prompts = self.get_prompts(cmd_ref);
                if prompts.is_empty() {
                    self.execute_cmd(cmd_ref, vec![], hwnd)
                } else {
                    self.mode = InputMode::ArgInput {
                        command: cmd_ref,
                        prompt_index: 0,
                        collected_args: Vec::new(),
                    };
                    self.arg_buffer.clear();
                    self.cursor = 0;
                    self.selection = None;
                    WindowAction::Nothing
                }
            }
            InputMode::ArgInput { .. } => self.advance_arg(hwnd),
        }
    }

    pub fn escape(&mut self) {
        match self.mode {
            InputMode::ArgInput { .. } => {
                self.mode = InputMode::Query;
                self.arg_buffer.clear();
                self.cursor = self.query.len();
                self.selection = None;
            }
            InputMode::Query => {}
        }
    }

    pub fn escape_should_hide(&self) -> bool {
        matches!(self.mode, InputMode::Query)
    }

    // ── internals ─────────────────────────────────────────────────────────────

    fn advance_arg(&mut self, hwnd: isize) -> WindowAction {
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
        self.cursor = 0;
        self.selection = None;
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
                return self.execute_cmd(command, args, hwnd);
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

    fn execute_cmd(&self, cmd_ref: CommandRef, args: Vec<String>, hwnd: isize) -> WindowAction {
        match cmd_ref {
            CommandRef::BuiltIn(idx) => {
                executor::run_builtin(&BUILT_INS[idx].action, &args, 0, hwnd)
            }
            CommandRef::User(idx) => {
                executor::run_user(&self.user_commands[idx].action, &args, 0, hwnd)
            }
        }
    }

    fn rebuild_results(&mut self) {
        self.results = search::build_results(&self.query, &self.user_commands);
    }
}
