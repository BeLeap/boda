use std::{
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
    pub fn new(cli: Cli) -> Manager {
        let state = state::State::new(cli);

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
                state.ui.vertical_scroll += 1;
            }
        }
    }

    fn handle_command_action(&self, command_action: action::Command) {
        let mut state = self.state.write().unwrap();
        match command_action {
            action::Command::RunResult(command_result) => {
                state.global.result = command_result;
                state.command.running_count -= 1;
            }
            action::Command::StartRun(t) => {
                state.command.prev_tick = t;
                state.command.running_count += 1;
            }
        }
    }
}
