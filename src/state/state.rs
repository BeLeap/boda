use crate::Cli;

#[derive(Debug, Clone)]
pub struct State {
    pub running: bool,

    pub command: Vec<String>,
    pub interval: f64,
    pub concurrency: u8,

    pub result: CommandResult,

    pub ui: Ui,
}

impl State {
    pub fn new(cli: Cli) -> State {
        State {
            running: true,

            command: cli.command,
            interval: cli.interval,
            concurrency: cli.concurrency,

            result: CommandResult::default(),

            ui: Ui::default(),
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

#[derive(Debug, Clone, Default)]
pub struct Ui {
    pub vertical_scroll: usize,
}
