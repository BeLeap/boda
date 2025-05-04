use std::{
    fs::{File, OpenOptions},
    io::Write,
    sync::{LazyLock, Mutex},
};

pub static LOGGER: LazyLock<Logger> = LazyLock::new(|| Logger::new("/tmp/boda.log"));

#[derive(Debug)]
pub struct Logger {
    file: Mutex<File>,
}

impl Logger {
    fn new(path: &'static str) -> Logger {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .expect("cannot open log file");

        return Logger {
            file: Mutex::new(file),
        };
    }

    pub fn log<T: std::fmt::Display>(&self, level: Level, line: T) {
        let mut file = self.file.lock().unwrap();
        let now = chrono::Local::now();

        writeln!(file, "[{}] level={} {}", now, level, line).expect("unable to write log");
    }

    pub fn debug<T: std::fmt::Display>(&self, line: T) {
        self.log(Level::Debug, line)
    }

    pub fn info<T: std::fmt::Display>(&self, line: T) {
        self.log(Level::Info, line)
    }

    pub fn warn<T: std::fmt::Display>(&self, line: T) {
        self.log(Level::Warn, line)
    }

    pub fn error<T: std::fmt::Display>(&self, line: T) {
        self.log(Level::Error, line)
    }
}

pub enum Level {
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Level::Debug => "DEBUG",
                Level::Info => "INFO",
                Level::Warn => "WARN",
                Level::Error => "ERROR",
            },
        )
    }
}
