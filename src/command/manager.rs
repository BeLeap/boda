use std::{
    env,
    process::Command,
    sync::{Arc, RwLock},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use crossbeam_channel::{select, tick, unbounded};
use log::{debug, error};

use crate::state::{action, state};

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
        debug!("run!");
        let shell = self.shell.clone();
        let command_tx = self.command_tx.clone();
        let command = {
            let state = state.read().unwrap();
            state.global.command.clone()
        };

        thread::spawn(move || {
            let now = chrono::Local::now();
            command_tx.send(action::Command::StartRun(t, now)).unwrap();

            let command = command.join(" ");
            let result = Command::new(shell)
                .args(["-c"])
                .arg(command)
                .output()
                .unwrap();

            let stdout = String::from_utf8_lossy(&result.stdout).to_string();
            let stderr = String::from_utf8_lossy(&result.stderr).to_string();
            let status = result.status;

            if let Err(e) = command_tx.send(action::Command::RunResult(
                now,
                stdout,
                stderr,
                status.code().unwrap() as u8,
            )) {
                error!("error send command result: {}", e);
            }

            debug!("run completed!");
        });
    }

    pub fn run(self, state: Arc<RwLock<state::State>>) -> JoinHandle<()> {
        thread::spawn(move || {
            let tick_duration = {
                let state = state.read().unwrap();
                state.command.tick
            };
            let ticker = tick(tick_duration);
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
