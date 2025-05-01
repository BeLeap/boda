mod command;
mod error;

use clap::Parser;
use crossbeam_channel::{Sender, bounded, select, tick};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, poll, read},
    execute, queue, style, terminal,
    tty::IsTty,
};
use ratatui::{
    DefaultTerminal, Frame,
    style::Stylize,
    text::Line,
    widgets::{Block, Paragraph},
};
use std::{
    env,
    io::{self, Write},
    thread,
    time::Duration,
};

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

    color_eyre::install().expect("unable to install color_eyre");
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result

    // let stdin = io::stdin();
    // if !stdin.is_tty() {
    //     return Err(error::BodaError::Custom(
    //         "Not an interactive terminal".to_string(),
    //     ));
    // }

    // let mut w = io::stdout();
    // execute!(w, terminal::EnterAlternateScreen)?;
    // terminal::enable_raw_mode()?;

    // let tick = tick(Duration::from_millis((interval * 1000.0).round() as u64));
    // let (end_tx, end_rx) = bounded::<bool>(1);

    // let key_handler = thread::spawn(move || {
    //     handle_keys(end_tx).unwrap();
    // });

    // loop {
    //     select! {
    //         recv(end_rx) -> _ => {
    //             execute!(
    //                 w,
    //                 cursor::SetCursorStyle::DefaultUserShape,
    //             )?;
    //             break;
    //         },
    //         recv(tick) -> _ => {
    //             let now = chrono::Local::now();
    //             let output = match command::run(shell.clone(), cli.command.clone()) {
    //                 Ok(output) => output,
    //                 Err(e) => format!("error: {}", e),
    //             };
    //             queue!(
    //                 w,
    //                 style::ResetColor,
    //                 terminal::Clear(terminal::ClearType::All),
    //                 cursor::Hide,
    //                 cursor::MoveTo(0, 0),
    //                 style::Print(now),
    //                 cursor::MoveToNextLine(1),
    //                 style::Print("==============".to_string()),
    //             )?;

    //             for line in output.lines() {
    //                 queue!(
    //                     w,
    //                     cursor::MoveToNextLine(1),
    //                     style::Print(line.to_string()),
    //                 )?;
    //             };

    //             w.flush()?;
    //         }
    //     }
    // }

    // execute!(
    //     w,
    //     style::ResetColor,
    //     cursor::Show,
    //     terminal::LeaveAlternateScreen
    // )?;
    // terminal::disable_raw_mode()?;
    // if let Err(_) = key_handler.join() {
    //     return Err(error::BodaError::Custom(
    //         "failed to join key_handler".to_string(),
    //     ));
    // };

    // Ok(())
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

/// The main application which holds the state and logic of the application.
#[derive(Debug, Default)]
pub struct App {
    /// Is the application running?
    running: bool,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> error::BodaResult<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    ///
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/main/ratatui-widgets/examples>
    fn render(&mut self, frame: &mut Frame) {
        let title = Line::from("Ratatui Simple Template")
            .bold()
            .blue()
            .centered();
        let text = "Hello, Ratatui!\n\n\
            Created using https://github.com/ratatui/templates\n\
            Press `Esc`, `Ctrl-C` or `q` to stop running.";
        frame.render_widget(
            Paragraph::new(text)
                .block(Block::bordered().title(title))
                .centered(),
            frame.area(),
        )
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> error::BodaResult<()> {
        match event::read()? {
            // it's important to check KeyEventKind::Press to avoid handling key release events
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            _ => {}
        }
        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
