use std::{
    sync::{Arc, RwLock},
    thread,
};

use crossbeam_channel::select;

use crate::util::log::LOGGER;

use super::{action, state};

#[derive(Debug)]
pub struct Manager {
    pub state: Arc<RwLock<state::State>>,
}

impl Manager {
    pub fn new(initial_tick: f64) -> Manager {
        let mut state = state::State::default();
        state.tick = initial_tick;

        Manager {
            state: Arc::new(RwLock::new(state)),
        }
    }

    pub fn run(
        self,
        action_rx: crossbeam_channel::Receiver<action::Action>,
        command_rx: crossbeam_channel::Receiver<String>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            loop {
                select! {
                    recv(action_rx) -> action_recv => {
                        if let Ok(action) = action_recv {
                            {
                                let mut state = self.state.write().unwrap();
                                match action {
                                    action::Action::Quit => {
                                        LOGGER.log("received quit");

                                        state.running = false;
                                    },
                                }
                            }

                            let state = self.state.read().unwrap();
                            if state.running == false {
                                LOGGER.log("stopping state manager..");
                                break;
                            }
                        }
                    }
                    recv(command_rx) -> command_recv => {
                        if let Ok(command) = command_recv {
                            LOGGER.log(&command)
                        }
                    }
                }
            }
        })
    }
}
