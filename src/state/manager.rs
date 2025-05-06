use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
    thread,
};

use crossbeam_channel::select;
use log::{debug, info};

use crate::Cli;

use super::{action, state};

#[derive(Debug)]
pub struct Manager {
    pub state: Arc<RwLock<state::State>>,
}

impl Manager {
    pub fn new(cli: Cli, filepath: &PathBuf) -> Manager {
        let state = state::State::new(cli, filepath);

        Manager {
            state: Arc::new(RwLock::new(state)),
        }
    }

    pub fn run(
        self,
        ui_action_rx: crossbeam_channel::Receiver<action::Ui>,
        command_action_rx: crossbeam_channel::Receiver<action::Command>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            loop {
                select! {
                    recv(ui_action_rx) -> action_recv => {
                        if let Ok(action) = action_recv {
                            self.handle_ui_action(action);

                            let state = self.state.read().unwrap();
                            if state.global.running == false {
                                info!("stopping state manager..");
                                break;
                            }
                        }
                    }
                    recv(command_action_rx) -> action_recv => {
                        if let Ok(action) = action_recv {
                            self.handle_command_action(action);
                        }
                    }
                }
            }
        })
    }

    fn handle_ui_action(&self, ui_action: action::Ui) {
        let mut state = self.state.write().unwrap();
        match ui_action {
            action::Ui::Quit => {
                debug!("received quit");

                state.global.running = false;
            }
            action::Ui::ScrollUp => {
                if state.ui.vertical_scroll > 0 {
                    state.ui.vertical_scroll -= 1;
                }
            }
            action::Ui::ScrollDown => {
                let length = match state
                    .global
                    .get_target_command_result(&state.ui.target_command)
                {
                    Some(r) => r.get_content().len(),
                    _ => return,
                };

                if (length - 1) as u16 <= state.ui.vertical_scroll {
                    return;
                }

                state.ui.vertical_scroll += 1;
            }
            action::Ui::ToggleShowHistory => {
                state.ui.show_history = !state.ui.show_history;
            }
            action::Ui::ToggleShowHelp => {
                state.ui.show_help = !state.ui.show_help;
            }
            action::Ui::SelectNext => match state.ui.target_command {
                state::TargetCommand::Latest => {
                    state.ui.target_command =
                        state::TargetCommand::Target(state.global.get_history()[0].id);
                }
                state::TargetCommand::Target(id) => {
                    let id = if id == 0 { 0 } else { id - 1 };
                    state.ui.target_command = state::TargetCommand::Target(id);
                }
            },
            action::Ui::SelectPrev => match state.ui.target_command {
                state::TargetCommand::Latest => {}
                state::TargetCommand::Target(id) => {
                    if state.global.get_history()[0].id == id {
                        state.ui.target_command = state::TargetCommand::Latest
                    } else {
                        state.ui.target_command = state::TargetCommand::Target(id + 1)
                    }
                }
            },
            action::Ui::SelectLatest => {
                state.ui.target_command = state::TargetCommand::Latest;
            }
        }
    }

    fn handle_command_action(&self, command_action: action::Command) {
        let mut state = self.state.write().unwrap();
        match command_action {
            action::Command::RunResult(timestamp, stdout, stderr, status) => {
                state
                    .global
                    .record_command_result(timestamp, stdout, stderr, status);
                state.command.running_count -= 1;
            }
            action::Command::StartRun(t, timestamp) => {
                state.global.record_command(timestamp);
                state.command.prev_tick = t;
                state.command.running_count += 1;
            }
        }
    }
}
