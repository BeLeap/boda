use std::time::Instant;

use crate::Cli;

#[derive(Debug, Clone)]
pub struct State {
    pub global: Global,
    pub ui: Ui,
}

impl State {
    pub fn new(cli: Cli) -> State {
        State {
            global: Global::new(cli),
            ui: Ui::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Global {
    pub running: bool,

    pub command: Vec<String>,
    pub interval: f64,
    pub concurrency: u8,

    pub result: CommandResult,
}

impl Global {
    pub fn new(cli: Cli) -> Global {
        Global {
            running: true,

            command: cli.command,
            interval: cli.interval,
            concurrency: cli.concurrency,

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

#[derive(Debug, Clone, Default)]
pub struct Ui {
    pub vertical_scroll: usize,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub prev_tick: Instant,
    pub running_count: u8,
}

impl Default for Command {
    fn default() -> Self {
        Command {
            prev_tick: Instant::now(),
            running_count: 0u8,
        }
    }
}
