mod error;

use clap::Parser;
use crossbeam_channel::{Sender, bounded, select, tick};
use crossterm::{
    cursor,
    event::{Event, KeyCode, KeyEvent, poll, read},
    execute, style, terminal,
    tty::IsTty,
};
use std::{io, thread, time::Duration};

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

    let stdin = io::stdin();
    if !stdin.is_tty() {
        return Err(error::BodaError::Custom(
            "Not an interactive terminal".to_string(),
        ));
    }

    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;

    let tick = tick(Duration::from_millis((interval * 1000.0).round() as u64));
    let (end_tx, end_rx) = bounded::<bool>(1);

    let key_handler = thread::spawn(move || {
        handle_keys(end_tx).unwrap();
    });

    let mut idx = 0;
    loop {
        select! {
            recv(end_rx) -> _ => {
                execute!(
                    stdout,
                    cursor::SetCursorStyle::DefaultUserShape,
                )?;
                break;
            },
            recv(tick) -> _ => {
                idx += 1;
                execute!(
                    stdout,
                    style::ResetColor,
                    terminal::Clear(terminal::ClearType::All),
                    cursor::Hide,
                    cursor::MoveTo(0, 0),
                    style::Print(idx.to_string()),
                )?;
            }
        }
    }

    execute!(
        stdout,
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
