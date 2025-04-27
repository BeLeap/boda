use std::process::Command;

use crate::error;

pub fn run(shell: String, command: Vec<String>) -> error::BodaResult<String> {
    let command = command.join(" ");
    let output = Command::new(shell).arg("-c").arg(command).output()?;

    return Ok(String::from_utf8_lossy(&output.stdout).to_string());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let output = run(
            "/bin/zsh".to_string(),
            vec!["curl".to_string(), "google.com".to_string()],
        )
        .unwrap();

        assert_eq!("", output);
    }
}
