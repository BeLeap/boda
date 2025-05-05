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
            let running_cnt = Arc::new(RwLock::new(1u8));

            let run = |command: Vec<String>| {
                LOGGER.debug("run!");
                let shell = self.shell.clone();
                let command_tx = self.command_tx.clone();
                let running_cnt = running_cnt.clone();

                thread::spawn(move || {
                    let now = chrono::Local::now();
                    let command = command.join(" ");
                    let output = Command::new(shell).arg("-c").arg(command).output().unwrap();

                    let result = String::from_utf8_lossy(&output.stdout).to_string();

                    command_tx
                        .send(action::Command::Append(CommandResult {
                            timestamp: now,
                            stdout: result,
                        }))
                        .unwrap();
                    LOGGER.debug("run completed!");
                    {
                        let mut running_cnt = running_cnt.write().unwrap();
                        *running_cnt -= 1;
                    }
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

                            let (interval, concurrency) = {
                                let state = state.read().unwrap();
                                (state.global.interval, state.global.concurrency)
                            };
                            let now_running = {
                                let running_cnt = running_cnt.read().unwrap();
                                *running_cnt
                            };

                            LOGGER.debug(format!("{:#?}", interval));
                            LOGGER.debug(format!("now_running: {:#?}", now_running));
                            if tick_diff.as_millis() > (interval * 1000.0) as u128 && now_running < concurrency {
                                LOGGER.debug("prepare run");

                                {
                                    LOGGER.debug("acquiring running_cnt write lock");
                                    let mut running_cnt = running_cnt.write().unwrap();
                                    LOGGER.debug("acquired running_cnt write lock");
                                    *running_cnt += 1;
                                }
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
