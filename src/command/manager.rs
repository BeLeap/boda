use std::{
    env,
    process::Command,
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crossbeam_channel::{select, tick, unbounded};

use crate::{
    state::{
        action,
        state::{self, CommandResult},
    },
    util::log::LOGGER,
};

pub struct Manager {
    command_tx: crossbeam_channel::Sender<action::Command>,

    shell: String,
}

impl Manager {
    pub fn new() -> (Manager, crossbeam_channel::Receiver<action::Command>) {
        let shell = match env::var("SHELL") {
            Ok(s) => s,
            Err(_) => "/bin/sh".to_string(),
        };

        let (tx, rx) = unbounded::<action::Command>();
        (
            Manager {
                command_tx: tx,

                shell,
            },
            rx,
        )
    }

    pub fn execute(&self, t: Instant, state: &Arc<RwLock<state::State>>) {
        LOGGER.debug("run!");
        let shell = self.shell.clone();
        let command_tx = self.command_tx.clone();
        let command = {
            let state = state.read().unwrap();
            state.global.command.clone()
        };

        thread::spawn(move || {
            command_tx.send(action::Command::StartRun(t)).unwrap();
            let now = chrono::Local::now();

            let command = command.join(" ");
            let output = Command::new(shell).arg("-c").arg(command).output().unwrap();

            let result = String::from_utf8_lossy(&output.stdout).to_string();

            command_tx
                .send(action::Command::RunResult(CommandResult {
                    timestamp: now,
                    stdout: result,
                }))
                .unwrap();
            LOGGER.debug("run completed!");
        });
    }

    pub fn run(self, state: Arc<RwLock<state::State>>) -> JoinHandle<()> {
        thread::spawn(move || {
            let ticker = tick(Duration::from_millis(100));
            // NOTE: Run at first
            self.execute(Instant::now(), &state);

            loop {
                select! {
                    recv(ticker) -> ticker_recv => {
                        if let Ok(t) = ticker_recv {
                            {
                                let state = state.read().unwrap();
                                if !state.global.running {
                                    break;
                                }
                            }


                            let can_run = {
                                let state = state.read().unwrap();
                                state.can_run(t)
                            };

                            if can_run {
                                self.execute(t, &state);
                            }
                        }
                    }
                }
            }
        })
    }
}
