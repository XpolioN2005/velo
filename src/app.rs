use crate::command::{BUILT_INS, CommandRef, UserCommand};
use crate::config::load_user_commands;

pub struct AppState {
    pub query: String,
    pub focused: bool,
    pub selected: usize,
    pub user_commands: Vec<UserCommand>,
    pub results: Vec<CommandRef>, // filtered indices, rebuilt on every keystroke
}

impl AppState {
    pub fn new() -> Self {
        let user_commands = load_user_commands();
        let mut state = Self {
            query: String::new(),
            focused: true,
            selected: 0,
            user_commands,
            results: Vec::new(),
        };
        state.rebuild_results();
        state
    }

    pub fn push_char(&mut self, c: char) {
        if self.query.len() < 100 {
            self.query.push(c);
            self.selected = 0;
            self.rebuild_results();
        }
    }

    pub fn pop_char(&mut self) {
        self.query.pop();
        self.selected = 0;
        self.rebuild_results();
    }

    pub fn clear_query(&mut self) {
        self.query.clear();
        self.selected = 0;
        self.rebuild_results();
    }

    pub fn select_next(&mut self) {
        if !self.results.is_empty() {
            self.selected = (self.selected + 1).min(self.results.len() - 1);
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    // Rebuild results from current query — called on every query change.
    // Empty query = show all. Non-empty = simple substring match for now
    // (fuzzy in Step 6).
    fn rebuild_results(&mut self) {
        self.results.clear();

        if self.query.is_empty() {
            return; // show nothing until user types
        }
        let q = self.query.to_lowercase();

        for (i, cmd) in BUILT_INS.iter().enumerate() {
            if q.is_empty() || matches_query(cmd.name, cmd.aliases, &q) {
                self.results.push(CommandRef::BuiltIn(i));
            }
        }

        for (i, cmd) in self.user_commands.iter().enumerate() {
            let aliases: Vec<&str> = cmd.aliases.iter().map(|s| s.as_str()).collect();
            if q.is_empty() || matches_query(&cmd.name, &aliases, &q) {
                self.results.push(CommandRef::User(i));
            }
        }
    }
}

fn matches_query(name: &str, aliases: &[&str], q: &str) -> bool {
    name.to_lowercase().contains(q) || aliases.iter().any(|a| a.to_lowercase().contains(q))
}
