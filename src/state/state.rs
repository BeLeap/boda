use std::time::Instant;

#[derive(Debug, Clone)]
pub struct State {
    pub running: bool,

    pub interval: f64,
    pub concurrency: u8,

    pub result: CommandResult,
}

impl Default for State {
    fn default() -> Self {
        State {
            running: true,

            interval: 0.0,
            concurrency: 0,

            result: CommandResult::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub stdout: String,
}

impl Default for CommandResult {
    fn default() -> Self {
        CommandResult {
            timestamp: chrono::Local::now(),
            stdout: "".to_string(),
        }
    }
}

impl std::fmt::Display for CommandResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}
