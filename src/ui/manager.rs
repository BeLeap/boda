use std::{
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use crossbeam_channel::{select, tick, unbounded};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, poll};
use log::{debug, error};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Margin},
    style::{Style, Stylize},
    text::Text,
    widgets::{Block, Paragraph},
};

use crate::{error::BodaResult, state};

#[derive(Debug)]
pub struct Manager {
    action_tx: crossbeam_channel::Sender<state::action::Ui>,
}

impl Manager {
    pub fn new() -> (Manager, crossbeam_channel::Receiver<state::action::Ui>) {
        let (tx, rx) = unbounded::<state::action::Ui>();

        (Manager { action_tx: tx }, rx)
    }
}

impl Manager {
    pub fn run(mut self, state: Arc<RwLock<state::state::State>>) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut terminal = setup_terminal();
            let ticker = tick(Duration::from_millis(100));

            loop {
                select! {
                    recv(ticker) -> _ => {
                        let state = state.read().unwrap();
                        if !state.global.running {
                            cleanup_terminal();
                            break;
                        }

                        terminal.draw(|frame| self.render(frame, &state)).unwrap();
                        if poll(Duration::from_secs(0)).unwrap() {
                            self.handle_crossterm_events().unwrap()
                        }
                    }
                }
            }
        })
    }

    fn handle_crossterm_events(&self) -> BodaResult<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            _ => {}
        }
        Ok(())
    }

    fn on_key_event(&self, key: KeyEvent) {
        debug!("key event");
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
                debug!("sending quit");
                if let Err(_) = self.action_tx.send(state::action::Ui::Quit) {
                    error!("error on send")
                }
                debug!("sent quit");
            }
            (_, KeyCode::Char('j')) => {
                self.action_tx.send(state::action::Ui::ScrollDown).unwrap();
            }
            (_, KeyCode::Char('k')) => {
                self.action_tx.send(state::action::Ui::ScrollUp).unwrap();
            }
            (_, KeyCode::Char(' ')) => {
                self.action_tx
                    .send(state::action::Ui::ToggleHistory)
                    .unwrap();
            }
            // Add other key handlers here.
            _ => {}
        }
    }

    fn render(&mut self, frame: &mut Frame, state: &state::state::State) {
        let result = state.global.last_command_result();
        let show_history = state.ui.show_history;

        let area = frame.area();
        let rows =
            Layout::vertical([Constraint::Length(3), Constraint::Percentage(100)]).split(area);

        let heading_chunks = Layout::horizontal([
            Constraint::Percentage(10),
            Constraint::Percentage(70),
            Constraint::Percentage(20),
        ])
        .split(rows[0]);

        let layout = if show_history {
            vec![Constraint::Percentage(80), Constraint::Percentage(20)]
        } else {
            vec![Constraint::Percentage(100)]
        };
        let content_chunks = Layout::horizontal(layout).split(rows[1]);

        frame.render_widget(
            Paragraph::new(format!("{}", state.global.interval)).block(
                Block::bordered()
                    .border_style(Style::new().gray())
                    .title("Every")
                    .title_style(Style::new().gray()),
            ),
            heading_chunks[0],
        );
        frame.render_widget(
            Paragraph::new(state.global.command.join(" ")).block(
                Block::bordered()
                    .border_style(Style::new().gray())
                    .title("Command")
                    .title_style(Style::new().gray()),
            ),
            heading_chunks[1],
        );
        frame.render_widget(
            Paragraph::new(match &result {
                Some(r) => format!("{}", r.timestamp),
                None => "Running".to_string(),
            })
            .block(
                Block::bordered()
                    .border_style(Style::new().gray())
                    .title("Timestamp")
                    .title_style(Style::new().gray()),
            ),
            heading_chunks[2],
        );

        if let Some(result) = result {
            frame.render_widget(
                Paragraph::new(result.stdout.clone().split("\r").collect::<String>())
                    .scroll(((state.ui.vertical_scroll as u16), 0)),
                content_chunks[0].inner(Margin {
                    horizontal: 1,
                    vertical: 0,
                }),
            );
        }

        if show_history {
            frame.render_widget(
                Block::bordered().border_style(Style::new().gray()),
                content_chunks[1],
            );

            let timestamps = state.global.get_history();

            let constraints = vec![Constraint::Length(1); timestamps.len()];
            let chunks = Layout::vertical(constraints).split(content_chunks[1].inner(Margin {
                horizontal: 1,
                vertical: 1,
            }));

            for (idx, timestamp) in timestamps.iter().enumerate() {
                frame.render_widget(Text::raw(format!("{}", timestamp)), chunks[idx]);
            }
        }
    }
}

fn cleanup_terminal() {
    ratatui::restore();
}

fn setup_terminal() -> DefaultTerminal {
    color_eyre::install().expect("unable to install color_eyre");
    let terminal = ratatui::init();

    return terminal;
}
