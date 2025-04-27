mod error;

use clap::Parser;
use crossbeam_channel::{Sender, bounded, select, tick};
use crossterm::{
    cursor,
    event::{Event, KeyCode, KeyEvent, poll, read},
    execute, style, terminal,
    tty::IsTty,
};
use std::{env, io, process::Command, thread, time::Duration};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short = 'n', long)]
    interval: Option<f64>,

    #[arg(last = true)]
    command: Vec<String>,
}

fn main() -> error::BodaResult<()> {
    let cli = Cli::parse();

    let interval = match cli.interval {
        Some(i) => i,
        None => 1.0,
    };

    let shell = match env::var("SHELL") {
        Ok(s) => s,
        Err(_) => "/bin/sh".to_string(),
    };

    let stdin = io::stdin();
    if !stdin.is_tty() {
        return Err(error::BodaError::Custom(
            "Not an interactive terminal".to_string(),
        ));
    }

    let mut w = io::stdout();
    execute!(w, terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    let tick = tick(Duration::from_millis((interval * 1000.0).round() as u64));
    let (end_tx, end_rx) = bounded::<bool>(1);

    let key_handler = thread::spawn(move || {
        handle_keys(end_tx).unwrap();
    });

    loop {
        select! {
            recv(end_rx) -> _ => {
                execute!(
                    w,
                    cursor::SetCursorStyle::DefaultUserShape,
                )?;
                break;
            },
            recv(tick) -> _ => {
                let now = chrono::Local::now();
                let shell = shell.clone();
                let command = cli.command.clone().join(" ");
                let output = Command::new(shell)
                    .arg("-c")
                    .arg(command)
                    .output();

                match output {
                    Ok(output) => {
                        let stdout = String::from_utf8(output.stdout).expect("not an utf8 string");
                        execute!(
                            w,
                            style::ResetColor,
                            terminal::Clear(terminal::ClearType::All),
                            cursor::Hide,
                            cursor::MoveTo(0, 0),
                            style::Print(now),
                            cursor::MoveToNextLine(1),
                            style::Print(stdout),
                        )?;
                    },
                    Err(e) => {
                        let out = format!("error: {}", e);
                        execute!(
                            w,
                            style::ResetColor,
                            terminal::Clear(terminal::ClearType::All),
                            cursor::Hide,
                            cursor::MoveTo(0, 0),
                            style::Print(out),
                        )?;
                    },
                };
            }
        }
    }

    execute!(
        w,
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen
    )?;
    terminal::disable_raw_mode()?;
    if let Err(_) = key_handler.join() {
        return Err(error::BodaError::Custom(
            "failed to join key_handler".to_string(),
        ));
    };

    Ok(())
}

fn handle_keys(end_tx: Sender<bool>) -> error::BodaResult<()> {
    loop {
        if poll(Duration::from_secs(1))? {
            if let Event::Key(KeyEvent {
                code,
                modifiers: _,
                kind: _,
                state: _,
            }) = read()?
            {
                match code {
                    KeyCode::Esc => {
                        end_tx.send(true)?;
                        break;
                    }
                    KeyCode::Char('q') => {
                        end_tx.send(true)?;
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    Ok(())
}
