use std::thread;

use crossbeam_channel::{select, unbounded};

use super::{action, state};

#[derive(Debug)]
pub struct Manager {
    state_tx: crossbeam_channel::Sender<state::State>,
}

impl Manager {
    pub fn new() -> (Manager, crossbeam_channel::Receiver<state::State>) {
        let (tx, rx) = unbounded::<state::State>();

        (Manager { state_tx: tx }, rx)
    }

    pub fn run(
        self,
        action_rx: crossbeam_channel::Receiver<action::Action>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut state = state::State::default();

            loop {
                select! {
                    recv(action_rx) -> action_recv => {
                        if let Ok(action) = action_recv {
                            match action {
                                action::Action::Quit => {
                                    state.running = false;
                                },
                            }

                            self.state_tx.send(state.clone()).expect("failed to send state");

                            if state.running == false {
                                break;
                            }
                        }
                    }
                }
            }
        })
    }
}
