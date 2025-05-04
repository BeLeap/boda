#[derive(Debug, Clone)]
pub struct State {
    pub running: bool,

    pub interval: f64,
    pub concurrency: u8,

    pub result: String,
}

impl Default for State {
    fn default() -> Self {
        State {
            running: true,

            interval: 0.0,
            concurrency: 0,

            result: "".to_string(),
        }
    }
}
