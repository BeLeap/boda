use std::{
    fs::File,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Instant,
};

use log::{debug, error, info};
use rusqlite::Connection;

use crate::{Cli, util};

#[derive(Debug)]
pub struct State {
    pub global: Global,
    pub ui: Ui,
    pub command: Command,
}

impl State {
    pub fn new(cli: Cli, filepath: &PathBuf) -> State {
        State {
            global: Global::new(cli, filepath),
            ui: Ui::default(),
            command: Command::default(),
        }
    }
}

impl State {
    pub fn can_run(&self, t: Instant) -> bool {
        let tick_diff = t - self.command.prev_tick;

        debug!("now running: {}", self.command.running_count);

        tick_diff.as_millis() > (self.global.interval * 1000.0) as u128
            && (self.command.running_count < self.global.concurrency)
    }
}

#[derive(Debug)]
pub struct Global {
    pub running: bool,

    pub command: Vec<String>,
    pub interval: f64,
    pub concurrency: u8,

    conn: Arc<Mutex<Connection>>,
}

impl Global {
    pub fn new(cli: Cli, filepath: &PathBuf) -> Global {
        File::create(filepath).unwrap();
        info!("db file at {:?}", filepath);
        let conn = Connection::open(filepath).unwrap();

        conn.execute(
            "CREATE TABLE command_result (
                id INTEGER PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                stdout TEXT,
                stderr TEXT,
                status INTEGER
            )",
            (),
        )
        .unwrap();

        let interval = if cli.interval < 0.5 {
            0.5
        } else {
            cli.interval
        };

        Global {
            running: true,

            command: cli.command,
            interval,
            concurrency: cli.concurrency,

            conn: Arc::new(Mutex::new(conn)),
        }
    }
}

impl Global {
    pub fn record_command(&self, timestamp: chrono::DateTime<chrono::Local>) {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO command_result (timestamp) VALUES (?1)",
            (timestamp.timestamp_millis(),),
        )
        .unwrap();
    }

    pub fn record_command_result(
        &self,
        timestamp: util::chrono::DateTime,
        stdout: String,
        stderr: String,
        status: u8,
    ) {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE command_result SET stdout=?1, stderr=?2, status=?3 WHERE timestamp=?4",
            (stdout, stderr, status, timestamp.timestamp_millis()),
        )
        .unwrap();
    }

    pub fn get_target_command_result(
        &self,
        target_command: &TargetCommand,
    ) -> Option<CommandResult> {
        match target_command {
            TargetCommand::Latest => self.last_command_result(),
            TargetCommand::Target(id) => self.get_command_result(*id),
        }
    }

    fn last_command_result(&self) -> Option<CommandResult> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = match conn.prepare(
            "SELECT timestamp, stdout, stderr, status FROM command_result WHERE status IS NOT NULL ORDER BY id DESC LIMIT 1",
        ) {
            Ok(stmt) => stmt,
            Err(e) => {
                error!("error on select: {}", e);
                return None;
            }
        };
        let result_iter = stmt
            .query_map([], |row| {
                Ok(CommandResult {
                    timestamp: chrono::DateTime::from_timestamp_millis(row.get(0).unwrap())
                        .unwrap()
                        .into(),
                    stdout: row.get(1).unwrap(),
                    stderr: row.get(2).unwrap(),
                    status: row.get(3).unwrap(),
                })
            })
            .unwrap();

        for result in result_iter {
            if let Ok(result) = result {
                return Some(result);
            }
        }
        None
    }

    fn get_command_result(&self, id: u32) -> Option<CommandResult> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = match conn.prepare(
            "SELECT timestamp, stdout, stderr, status FROM command_result WHERE status IS NOT NULL AND id=?1 ORDER BY id DESC LIMIT 1",
        ) {
            Ok(stmt) => stmt,
            Err(e) => {
                error!("error on select: {}", e);
                return None;
            }
        };
        let result_iter = stmt
            .query_map([id], |row| {
                Ok(CommandResult {
                    timestamp: chrono::DateTime::from_timestamp_millis(row.get(0).unwrap())
                        .unwrap()
                        .into(),
                    stdout: row.get(1).unwrap(),
                    stderr: row.get(2).unwrap(),
                    status: row.get(3).unwrap(),
                })
            })
            .unwrap();

        for result in result_iter {
            if let Ok(result) = result {
                return Some(result);
            }
        }
        None
    }

    pub fn get_history(&self) -> Vec<CommandResultSummary> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = match conn
            .prepare("SELECT id, timestamp, status FROM command_result ORDER BY id DESC")
        {
            Ok(stmt) => stmt,
            Err(e) => {
                error!("error on select: {}", e);
                return vec![];
            }
        };
        let result_iter = stmt
            .query_map([], |row| {
                Ok(CommandResultSummary {
                    id: row.get(0).unwrap(),
                    timestamp: chrono::DateTime::from_timestamp_millis(row.get(1).unwrap())
                        .unwrap()
                        .into(),
                    status: row.get(2).unwrap(),
                })
            })
            .unwrap();

        return result_iter.map(|it| it.unwrap()).collect();
    }
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub timestamp: util::chrono::DateTime,

    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub status: Option<u8>,
}

pub struct CommandResultSummary {
    pub id: u32,
    pub timestamp: util::chrono::DateTime,
    pub status: Option<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct Ui {
    pub show_history: bool,
    pub relative_history: bool,

    pub vertical_scroll: usize,

    pub show_help: bool,
    pub target_command: TargetCommand,
}

#[derive(Debug, Clone, Default)]
pub enum TargetCommand {
    #[default]
    Latest,
    Target(u32),
}

impl TargetCommand {
    pub fn is_target(&self, summary: &CommandResultSummary) -> bool {
        match self {
            TargetCommand::Latest => false,
            TargetCommand::Target(id) => *id == summary.id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Command {
    pub prev_tick: Instant,
    pub running_count: u8,
}

impl Default for Command {
    fn default() -> Self {
        Command {
            prev_tick: Instant::now(),
            running_count: 0u8,
        }
    }
}
