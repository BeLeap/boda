use std::time::Instant;

use super::state::CommandResult;

#[derive(Debug)]
pub enum Ui {
    Quit,

    ScrollDown,
    ScrollUp,
}

#[derive(Debug)]
pub enum Command {
    RunResult(CommandResult),
    StartRun(Instant),
}
