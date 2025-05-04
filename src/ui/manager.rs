use std::{thread, time::Duration};

use crossbeam_channel::{select, tick, unbounded};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, poll};
use ratatui::{DefaultTerminal, Frame, widgets::Paragraph};

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
    pub fn run(
        self,
        state_rx: crossbeam_channel::Receiver<state::state::State>,
    ) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut terminal = setup_terminal();
            let tick = tick(Duration::from_millis(100));

            loop {
                select! {
                    recv(state_rx) -> state_recv => {
                        if let Ok(state) = state_recv {
                            if !state.running {
                                cleanup_terminal();
                                break;
                            }
                        }
                    }
                    recv(tick) -> _ => {
                        terminal.draw(|frame| self.render(frame)).unwrap();
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
        LOGGER.log("key event");
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
                LOGGER.log("sending quit");
                if let Err(_) = self.action_tx.send(state::action::Action::Quit) {
                    LOGGER.log("error on send")
                }
                LOGGER.log("sent quit");
            }
            // Add other key handlers here.
            _ => {}
        }
    }

    fn render(&self, frame: &mut Frame) {
        frame.render_widget(Paragraph::new("Hello, World!"), frame.area());
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
