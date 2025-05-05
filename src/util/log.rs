use std::{
    fs::{File, OpenOptions},
    io::Write,
    sync::{LazyLock, Mutex},
};

static LOGGER: LazyLock<Logger> = LazyLock::new(|| Logger::new("/tmp/boda.log"));

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

    pub fn log<T: std::fmt::Display>(&self, level: log::Level, line: T) {
        let mut file = self.file.lock().unwrap();
        let now = chrono::Local::now();

        writeln!(file, "[{}] level={} {}", now, level, line).expect("unable to write log");
    }
}

impl log::Log for Logger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        LOGGER.log(record.level(), record.args());
    }

    fn flush(&self) {}
}
