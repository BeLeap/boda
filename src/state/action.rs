use std::time::Instant;

use crate::util;

#[derive(Debug)]
pub enum Ui {
    Quit,

    ToggleShowHistory,

    SelectNext,
    SelectPrev,
    SelectLatest,

    ToggleShowHelp,

    ScrollDown,
    ScrollUp,
}

#[derive(Debug)]
pub enum Command {
    RunResult(
        util::chrono::DateTime,
        util::chrono::DateTime,
        String,
        String,
        u8,
    ),
    StartRun(Instant, util::chrono::DateTime),
}
