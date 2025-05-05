use std::{
    fs::File,
    sync::{Arc, Mutex},
    time::Instant,
};

use log::{error, info};
use rusqlite::Connection;

use crate::Cli;

#[derive(Debug)]
pub struct State {
    pub global: Global,
    pub ui: Ui,
    pub command: Command,
}

impl State {
    pub fn new(cli: Cli) -> State {
        State {
            global: Global::new(cli),
            ui: Ui::default(),
            command: Command::default(),
        }
    }
}

impl State {
    pub fn can_run(&self, t: Instant) -> bool {
        let tick_diff = t - self.command.prev_tick;

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
    pub fn new(cli: Cli) -> Global {
        let tempdir = std::env::temp_dir();
        let path = tempdir.join("backup.sqlite");
        File::create(&path).unwrap();
        info!("db file at {:?}", path);
        let conn = Connection::open_with_flags(
            path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE | rusqlite::OpenFlags::SQLITE_OPEN_CREATE,
        )
        .unwrap();

        conn.execute(
            "CREATE TABLE command_result (
                id INTEGER PRIMARY KEY,
                timestamp INTEGER NOT NULL,
                stdout TEXT NOT NULL
            )",
            (),
        )
        .unwrap();

        Global {
            running: true,

            command: cli.command,
            interval: cli.interval,
            concurrency: cli.concurrency,

            conn: Arc::new(Mutex::new(conn)),
        }
    }
}

impl Global {
    pub fn append_command_result(&self, command_result: CommandResult) {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO command_result (timestamp, stdout) VALUES (?1, ?2)",
            (
                command_result.timestamp.timestamp_millis(),
                command_result.stdout,
            ),
        )
        .unwrap();
    }

    pub fn last_command_result(&self) -> Option<CommandResult> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = match conn
            .prepare("SELECT timestamp, stdout FROM command_result ORDER BY id DESC LIMIT 1")
        {
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
                })
            })
            .unwrap();

        for result in result_iter {
            if let Ok(result) = result {
                return Some(result);
            }
        }
        panic!("result not exists");
    }
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub stdout: String,
}

impl Default for CommandResult {
    fn default() -> Self {
        CommandResult {
            timestamp: chrono::Local::now(),
            stdout: "".to_string(),
        }
    }
}

impl std::fmt::Display for CommandResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self)
    }
}

#[derive(Debug, Clone, Default)]
pub struct Ui {
    pub vertical_scroll: usize,
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
