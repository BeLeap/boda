use std::time::Instant;

use super::state::CommandResult;

#[derive(Debug)]
pub enum Ui {
    Quit,

    ToggleShowHistory,
    ToggleRelativeHistory,

    ScrollDown,
    ScrollUp,
}

#[derive(Debug)]
pub enum Command {
    RunResult(CommandResult),
    StartRun(Instant),
}
