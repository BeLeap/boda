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
    ToggleLineNumbers,
    ToggleTailFollow(u16),

    ScrollDown(u16),
    ScrollUp(u16),
    ScrollHalfDown(u16),
    ScrollHalfUp(u16),
    ScrollToTop,
    ScrollToBottom(u16),
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
