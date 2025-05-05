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

    pub fn run(self, state: Arc<RwLock<state::State>>) -> JoinHandle<()> {
        thread::spawn(move || {
            let ticker = tick(Duration::from_millis(100));
            let mut prev_tick: Instant = Instant::now();

            let run = |command: Vec<String>| {
                LOGGER.debug("run!");
                let shell = self.shell.clone();
                let command_tx = self.command_tx.clone();

                thread::spawn(move || {
                    command_tx.send(action::Command::StartRun).unwrap();
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
            };
            let command = {
                let state = state.read().unwrap();
                state.global.command.clone()
            };
            run(command);

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


                            let tick_diff = t - prev_tick;
                            LOGGER.debug(format!("{:#?}", tick_diff));

                            let (interval, concurrency, command_state) = {
                                let state = state.read().unwrap();
                                (state.global.interval, state.global.concurrency, state.command.clone())
                            };
                            LOGGER.debug(format!("now running: {}", command_state.running_count));

                            if tick_diff.as_millis() > (interval * 1000.0) as u128 && command_state.running_count < concurrency {
                                LOGGER.debug("prepare run");

                                let command = {
                                    let state = state.read().unwrap();
                                    state.global.command.clone()
                                };
                                run(command);
                                prev_tick = t;
                            }
                        }
                    }
                }
            }
        })
    }
}
