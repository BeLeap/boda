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

    pub fn log<T: std::fmt::Display>(&self, line: T) {
        let mut file = self.file.lock().unwrap();
        let now = time::UtcDateTime::now();

        writeln!(file, "[{}] {}", now, line).expect("unable to write log");
    }
}
