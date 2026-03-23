#[derive(Clone)]
pub enum Action {
    Internal(InternalAction),

    Launch { program: String, args: Vec<String> },

    OpenUrl { url: String },

    Sequence(Vec<Step>),
    Parallel(Vec<Action>),
}

#[derive(Clone)]
pub enum InternalAction {
    Quit,
    Hide,
}

#[derive(Clone)]
pub struct Step {
    pub action: Action,
    pub wait: bool,
    pub stop_on_fail: bool,
}
