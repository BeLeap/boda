use std::{
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use crossbeam_channel::{select, tick, unbounded};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, poll};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Margin},
    style::{Style, Stylize},
    widgets::{Block, Paragraph},
};

use crate::{error, state, util::log::LOGGER};

#[derive(Debug)]
pub struct Manager {
    action_tx: crossbeam_channel::Sender<state::action::Action>,
}

impl Manager {
    pub fn new() -> (Manager, crossbeam_channel::Receiver<state::action::Action>) {
        let (tx, rx) = unbounded::<state::action::Action>();

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
                        if !state.running {
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

    fn handle_crossterm_events(&self) -> error::BodaResult<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            _ => {}
        }
        Ok(())
    }

    fn on_key_event(&self, key: KeyEvent) {
        LOGGER.debug("key event");
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
                LOGGER.debug("sending quit");
                if let Err(_) = self.action_tx.send(state::action::Action::Quit) {
                    LOGGER.error("error on send")
                }
                LOGGER.debug("sent quit");
            }
            (_, KeyCode::Char('j')) => {
                self.action_tx
                    .send(state::action::Action::ScrollDown)
                    .unwrap();
            }
            (_, KeyCode::Char('k')) => {
                self.action_tx
                    .send(state::action::Action::ScrollUp)
                    .unwrap();
            }
            // Add other key handlers here.
            _ => {}
        }
    }

    fn render(&mut self, frame: &mut Frame, state: &state::state::State) {
        let area = frame.area();
        let chunks =
            Layout::vertical([Constraint::Length(3), Constraint::Percentage(100)]).split(area);

        let heading_chunks =
            Layout::horizontal([Constraint::Percentage(10), Constraint::Percentage(90)])
                .split(chunks[0]);

        frame.render_widget(
            Paragraph::new(format!("{}", state.interval)).block(
                Block::bordered()
                    .border_style(Style::new().gray())
                    .title("Every")
                    .title_style(Style::new().gray()),
            ),
            heading_chunks[0],
        );
        frame.render_widget(
            Paragraph::new(state.command.join(" ")).block(
                Block::bordered()
                    .border_style(Style::new().gray())
                    .title("Command")
                    .title_style(Style::new().gray()),
            ),
            heading_chunks[1],
        );

        frame.render_widget(
            Paragraph::new(state.result.stdout.clone().split("\r").collect::<String>())
                .scroll(((state.vertical_scroll as u16), 0)),
            chunks[1].inner(Margin {
                horizontal: 1,
                vertical: 0,
            }),
        );
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
