use std::time::Duration;

pub type DateTime = chrono::DateTime<chrono::Local>;

pub fn format_duration(duration: Duration) -> String {
    if duration.as_secs() < 1 {
        format!("{}ms", duration.as_millis())
    } else {
        format!("{:.3}s", duration.as_secs_f64())
    }
}
