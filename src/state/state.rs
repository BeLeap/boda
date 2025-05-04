#[derive(Debug, Clone)]
pub struct State {
    pub running: bool,
    pub tick: f64,
}

impl Default for State {
    fn default() -> Self {
        State {
            running: true,
            tick: 0.0,
        }
    }
}
