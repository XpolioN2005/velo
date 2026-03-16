/// All runtime state for the palette.
/// Passed into the renderer on every WM_PAINT.
pub struct AppState {
    pub query: String,
    pub focused: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            query: String::new(),
            focused: true,
        }
    }

    /// Append a character to the query
    pub fn push_char(&mut self, c: char) {
        self.query.push(c);
    }

    /// Remove last character from query
    pub fn pop_char(&mut self) {
        self.query.pop();
    }

    /// Clear the query entirely
    pub fn clear_query(&mut self) {
        self.query.clear();
    }
}
