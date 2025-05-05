use std::{
    sync::{Arc, RwLock},
    thread,
};

use crossbeam_channel::select;

use crate::{Cli, util::log::LOGGER};

use super::{action, state};

#[derive(Debug)]
pub struct Manager {
    pub state: Arc<RwLock<state::State>>,
}

impl Manager {
    pub fn new(cli: Cli) -> Manager {
        let mut state = state::State::default();

        state.interval = cli.interval;
        state.concurrency = cli.concurrency;
        state.command = cli.command;

        Manager {
            state: Arc::new(RwLock::new(state)),
        }
    }

    pub fn run(
        self,
        ui_action_rx: crossbeam_channel::Receiver<action::Ui>,
        command_rx: crossbeam_channel::Receiver<state::CommandResult>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            loop {
                select! {
                    recv(ui_action_rx) -> action_recv => {
                        if let Ok(action) = action_recv {
                            {
                                let mut state = self.state.write().unwrap();
                                match action {
                                    action::Ui::Quit => {
                                        LOGGER.debug("received quit");

                                        state.running = false;
                                    },
                                    action::Ui::ScrollUp => {
                                        if state.vertical_scroll > 0 {
                                            state.vertical_scroll -= 1;
                                        }
                                    },
                                    action::Ui::ScrollDown => {
                                        state.vertical_scroll += 1;
                                    },
                                }
                            }

                            let state = self.state.read().unwrap();
                            if state.running == false {
                                LOGGER.info("stopping state manager..");
                                break;
                            }
                        }
                    }
                    recv(command_rx) -> command_recv => {
                        if let Ok(command) = command_recv {
                            {
                                let mut state = self.state.write().unwrap();
                                state.result = command;
                            };
                        }
                    }
                }
            }
        })
    }
}
