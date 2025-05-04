use std::{
    env,
    process::Command,
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crossbeam_channel::{select, tick, unbounded};

use crate::{state::state, util::log::LOGGER};

pub struct Manager {
    command_tx: crossbeam_channel::Sender<String>,

    shell: String,
    command: Vec<String>,
}

impl Manager {
    pub fn new(command: Vec<String>) -> (Manager, crossbeam_channel::Receiver<String>) {
        let shell = match env::var("SHELL") {
            Ok(s) => s,
            Err(_) => "/bin/sh".to_string(),
        };

        let (tx, rx) = unbounded::<String>();
        (
            Manager {
                command_tx: tx,

                shell,
                command,
            },
            rx,
        )
    }

    pub fn run(self, state: Arc<RwLock<state::State>>) -> JoinHandle<()> {
        thread::spawn(move || {
            let ticker = tick(Duration::from_millis(100));
            let mut prev_tick: Instant = Instant::now();

            select! {
                recv(ticker) -> ticker_recv => {
                    if let Ok(t) = ticker_recv {
                        let tick_diff = t - prev_tick;

                        let tick = {
                            let state = state.read().unwrap();
                            state.tick
                        };
                        if tick_diff.as_millis() > (tick * 1000.0) as u128 {
                            let result = self.execute();
                            if let Ok(_) = self.command_tx.send(result) {
                                prev_tick = t;
                            }
                        }
                    }
                }
            }
        })
    }

    fn execute(&self) -> String {
        let command = self.command.join(" ");
        let output = Command::new(self.shell.clone())
            .arg("-c")
            .arg(command)
            .output()
            .unwrap();

        return String::from_utf8_lossy(&output.stdout).to_string();
    }
}
