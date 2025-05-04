#[derive(Debug, Clone)]
pub struct State {
    pub running: bool,
}

impl Default for State {
    fn default() -> Self {
        State { running: true }
    }
}
