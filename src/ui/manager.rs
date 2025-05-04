use std::thread;

use crossbeam_channel::{select, unbounded};

use crate::state;

#[derive(Debug)]
pub struct Manager {
    action_tx: crossbeam_channel::Sender<state::action::Action>,
}

impl Manager {
    pub fn new() -> (Manager, crossbeam_channel::Receiver<state::action::Action>) {
        let (tx, rx) = unbounded::<state::action::Action>();

        (Manager { action_tx: tx }, rx)
    }

    pub fn run(
        &self,
        state_rx: crossbeam_channel::Receiver<state::state::State>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            loop {
                select! {
                    recv(state_rx) -> _ => {}
                }
            }
        })
    }
}
