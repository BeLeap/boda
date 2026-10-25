use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
    thread,
};

use crossbeam_channel::select;
use log::{debug, info};

use crate::Cli;

use super::{action, state};

#[derive(Debug, Clone)]
pub struct Manager {
    pub state: Arc<RwLock<state::State>>,
}

fn max_scroll(content_length: usize, viewport_height: u16) -> u16 {
    content_length
        .saturating_sub(usize::from(viewport_height))
        .min(usize::from(u16::MAX)) as u16
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
    ) -> (thread::JoinHandle<()>, thread::JoinHandle<()>) {
        let ui_manager = self.clone();
        let ui_handle = thread::spawn(move || {
            loop {
                select! {
                    recv(ui_action_rx) -> action_recv => {
                        match action_recv {
                            Ok(action) => ui_manager.handle_ui_action(action),
                            Err(_) => break,
                        }
                    }
                }

                let state = ui_manager.state.read().unwrap();
                if !state.global.running {
                    info!("stopping ui state manager..");
                    break;
                }
            }
        });

        let command_manager = self;
        let command_handle = thread::spawn(move || {
            loop {
                select! {
                    recv(command_action_rx) -> action_recv => {
                        match action_recv {
                            Ok(action) => command_manager.handle_command_action(action),
                            Err(_) => break,
                        }
                    }
                }
            }
        });

        (ui_handle, command_handle)
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
            action::Ui::ScrollDown(viewport_height) => {
                let length = match state
                    .global
                    .get_target_command_result(&state.ui.target_command)
                {
                    Some(r) => r.get_content().len(),
                    _ => return,
                };
                let max_scroll = max_scroll(length, viewport_height);
                state.ui.vertical_scroll =
                    state.ui.vertical_scroll.saturating_add(1).min(max_scroll);
            }
            action::Ui::ScrollHalfDown(viewport_height) => {
                let length = match state
                    .global
                    .get_target_command_result(&state.ui.target_command)
                {
                    Some(r) => r.get_content().len(),
                    _ => return,
                };

                let max_scroll = max_scroll(length, viewport_height);
                state.ui.vertical_scroll =
                    state.ui.vertical_scroll.saturating_add(10).min(max_scroll);
            }
            action::Ui::ScrollHalfUp => {
                state.ui.vertical_scroll = state.ui.vertical_scroll.saturating_sub(10);
            }
            action::Ui::ScrollToTop => {
                state.ui.vertical_scroll = 0;
            }
            action::Ui::ScrollToBottom(viewport_height) => {
                let length = match state
                    .global
                    .get_target_command_result(&state.ui.target_command)
                {
                    Some(r) => r.get_content().len(),
                    _ => return,
                };
                state.ui.vertical_scroll = max_scroll(length, viewport_height);
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
            action::Command::RunResult(start, end, stdout, stderr, status) => {
                state
                    .global
                    .record_command_result(start, end, stdout, stderr, status);
                state.command.running_count -= 1;
            }
            action::Command::StartRun(t, start) => {
                state.global.record_command(start);
                state.command.prev_tick = t;
                state.command.running_count += 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::max_scroll;

    #[test]
    fn max_scroll_accounts_for_viewport_height() {
        assert_eq!(max_scroll(100, 20), 80);
        assert_eq!(max_scroll(10, 20), 0);
        assert_eq!(max_scroll(usize::MAX, 0), u16::MAX);
    }
}
