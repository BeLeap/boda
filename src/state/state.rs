use std::{
    fs::File,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
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

        tick_diff > (self.global.interval - self.command.tick)
            && (self.command.running_count < self.global.concurrency)
    }
}

#[derive(Debug)]
pub struct Global {
    pub running: bool,

    pub command: Vec<String>,
    pub interval: Duration,
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
                start INTEGER NOT NULL,
                end INTEGER,
                stdout TEXT,
                stderr TEXT,
                status INTEGER
            )",
            (),
        )
        .unwrap();

        let interval = if cli.interval < 0.5 {
            Duration::from_millis(500)
        } else {
            Duration::from_secs_f64(cli.interval)
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
    pub fn record_command(&self, start: chrono::DateTime<chrono::Local>) {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO command_result (start) VALUES (?1)",
            (start.timestamp_millis(),),
        )
        .unwrap();
    }

    pub fn record_command_result(
        &self,
        start: util::chrono::DateTime,
        end: util::chrono::DateTime,
        stdout: String,
        stderr: String,
        status: u8,
    ) {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE command_result SET stdout=?1, stderr=?2, status=?3, end=?4 WHERE start=?5",
            (
                stdout,
                stderr,
                status,
                end.timestamp_millis(),
                start.timestamp_millis(),
            ),
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
            "SELECT start, stdout, stderr, status FROM command_result WHERE status IS NOT NULL ORDER BY id DESC LIMIT 1",
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
                    start: chrono::DateTime::from_timestamp_millis(row.get(0).unwrap())
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

    fn get_command_result(&self, id: u16) -> Option<CommandResult> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = match conn.prepare(
            "SELECT start, stdout, stderr, status FROM command_result WHERE status IS NOT NULL AND id=?1 ORDER BY id DESC LIMIT 1",
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
                    start: chrono::DateTime::from_timestamp_millis(row.get(0).unwrap())
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
            .prepare("SELECT id, start, end, status FROM command_result ORDER BY id DESC")
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
                    start: chrono::DateTime::from_timestamp_millis(row.get(1).unwrap())
                        .unwrap()
                        .into(),
                    end: match row.get::<usize, i64>(2) {
                        Ok(r) => Some(chrono::DateTime::from_timestamp_millis(r).unwrap().into()),
                        Err(_) => None,
                    },
                    status: row.get(3).unwrap(),
                })
            })
            .unwrap();

        return result_iter.map(|it| it.unwrap()).collect();
    }
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub start: util::chrono::DateTime,

    pub stdout: Option<String>,
    pub stderr: Option<String>,
    pub status: Option<u8>,
}

impl CommandResult {
    fn lines(input: &Option<String>) -> Vec<String> {
        if let Some(input) = input {
            input.lines().map(|line| line.to_string()).collect()
        } else {
            vec![]
        }
    }

    pub fn get_content(&self) -> Vec<String> {
        match self.status {
            Some(status) => {
                if status == 0 {
                    CommandResult::lines(&self.stdout)
                } else {
                    CommandResult::lines(&self.stderr)
                }
            }
            None => vec!["Running".to_string()],
        }
    }
}

pub struct CommandResultSummary {
    pub id: u16,
    pub start: util::chrono::DateTime,
    pub end: Option<util::chrono::DateTime>,
    pub status: Option<u8>,
}

#[derive(Debug, Clone, Default)]
pub struct Ui {
    pub show_history: bool,

    pub vertical_scroll: u16,

    pub show_help: bool,
    pub target_command: TargetCommand,
}

#[derive(Debug, Clone, Default)]
pub enum TargetCommand {
    #[default]
    Latest,
    Target(u16),
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
    pub tick: Duration,
    pub prev_tick: Instant,
    pub running_count: u8,
}

impl Default for Command {
    fn default() -> Self {
        Command {
            tick: Duration::from_millis(10),
            prev_tick: Instant::now(),
            running_count: 0u8,
        }
    }
}
