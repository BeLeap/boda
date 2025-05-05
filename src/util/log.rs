use std::{
    fs::{File, OpenOptions},
    io::Write,
    sync::{LazyLock, Mutex},
};

static LOGGER: LazyLock<FileLogger> = LazyLock::new(|| FileLogger::new("/tmp/boda.log"));

#[derive(Debug)]
pub struct FileLogger {
    file: Mutex<File>,
}

impl FileLogger {
    fn new(path: &'static str) -> FileLogger {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .expect("cannot open log file");

        return FileLogger {
            file: Mutex::new(file),
        };
    }

    pub fn log<T: std::fmt::Display>(&self, level: log::Level, line: T) {
        let mut file = self.file.lock().unwrap();
        let now = chrono::Local::now();

        writeln!(file, "[{}] level={} {}", now, level, line).expect("unable to write log");
    }
}

struct Logger;

impl log::Log for Logger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        LOGGER.log(record.level(), record.args());
    }

    fn flush(&self) {}
}

pub fn setup() {
    log::set_logger(&Logger)
        .map(|()| log::set_max_level(log::LevelFilter::Trace))
        .unwrap();
}
