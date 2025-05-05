use chrono::TimeDelta;

pub type DateTime = chrono::DateTime<chrono::Local>;

pub fn human_readable_delta(td: TimeDelta) -> String {
    let diff = td.num_milliseconds();

    if diff.abs() < 1000 {
        return format!("{}ms", diff);
    }

    if diff.abs() < 60 * 1000 {
        return format!("{}s", diff / 1000);
    }

    let min = diff / (60 * 1000);
    let remainder = diff % (60 * 1000);
    return format!("{}m {}s", min, remainder.abs() / 1000);
}
