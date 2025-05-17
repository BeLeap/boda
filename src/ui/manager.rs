use std::{
    sync::{Arc, RwLock},
    thread,
    time::Duration,
};

use crossbeam_channel::{bounded, select, tick};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, poll};
use log::{debug, error};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Margin},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph},
};

use crate::{error::BodaResult, state};

#[derive(Debug)]
pub struct Manager {
    action_tx: crossbeam_channel::Sender<state::action::Ui>,
}

impl Manager {
    pub fn new() -> (Manager, crossbeam_channel::Receiver<state::action::Ui>) {
        let (tx, rx) = bounded::<state::action::Ui>(3);

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
                    .send(state::action::Ui::ToggleShowHistory)
                    .unwrap();
            }
            (_, KeyCode::Char('?')) => {
                self.action_tx
                    .send(state::action::Ui::ToggleShowHelp)
                    .unwrap();
            }
            (_, KeyCode::Char('p')) => {
                self.action_tx.send(state::action::Ui::SelectPrev).unwrap();
            }
            (_, KeyCode::Char('n')) => {
                self.action_tx.send(state::action::Ui::SelectNext).unwrap();
            }
            (_, KeyCode::Char('l')) => {
                self.action_tx
                    .send(state::action::Ui::SelectLatest)
                    .unwrap();
            }
            // Add other key handlers here.
            _ => {}
        }
    }

    fn render(&mut self, frame: &mut Frame, state: &state::state::State) {
        if state.ui.show_help {
            frame.render_widget(
                Paragraph::new(
                    "Keybindings

?: Help
q: Quit
j: Scroll Down
k: Scroll Up
<Space>: Show History
p: Show previous
n: Show next
l: Show latest",
                ),
                frame.area(),
            );

            return;
        }

        let result = state
            .global
            .get_target_command_result(&state.ui.target_command);
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
            Paragraph::new(if state.global.interval < 1.0 {
                format!("{}ms", (state.global.interval * 1000.0) as u128)
            } else {
                format!("{}s", state.global.interval)
            })
            .block(
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
            let (content, style) = (
                Text::from(
                    result
                        .get_content()
                        .iter()
                        .map(|line| Line::from(line.clone()))
                        .collect::<Vec<Line>>(),
                ),
                match result.status {
                    Some(0) => Style::default(),
                    Some(_) => Style::default().fg(Color::Red),
                    None => Style::default().fg(Color::Gray),
                },
            );

            frame.render_widget(
                Paragraph::new(content)
                    .style(style)
                    .scroll((state.ui.vertical_scroll, 0)),
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

            let history = state.global.get_history();
            let lines = history
                .iter()
                .map(|summary| {
                    let timestamp = (
                        format!("{}", summary.timestamp.time()),
                        if state.ui.target_command.is_target(summary) {
                            Style::default().bg(Color::DarkGray)
                        } else {
                            Style::default()
                        },
                    );
                    let status = match summary.status {
                        Some(0) => ("0".to_string(), Style::default().fg(Color::Green)),
                        Some(s) => (format!("{}", s), Style::default().fg(Color::Red)),
                        None => ("Running".to_string(), Style::default().fg(Color::Gray)),
                    };

                    Line::from(vec![
                        Span::styled(timestamp.0, timestamp.1),
                        Span::raw(" "),
                        Span::styled(status.0, status.1),
                    ])
                })
                .collect::<Vec<Line>>();
            let text = Text::from(lines);

            let scroll_offset = match state.ui.target_command {
                state::state::TargetCommand::Latest => 0,
                state::state::TargetCommand::Target(id) => {
                    let height = content_chunks[1].height.saturating_sub(2); // Margin 고려
                    (history.len() as u16 - id).saturating_sub(height / 2)
                }
            };

            frame.render_widget(
                Paragraph::new(text).scroll((scroll_offset, 0)),
                content_chunks[1].inner(Margin {
                    horizontal: 1,
                    vertical: 1,
                }),
            );
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
