use super::state::CommandResult;

#[derive(Debug)]
pub enum Ui {
    Quit,

    ScrollDown,
    ScrollUp,
}

#[derive(Debug)]
pub enum Command {
    Append(CommandResult),
    IncreaseRunning,
    DecreaseRunning,
}
