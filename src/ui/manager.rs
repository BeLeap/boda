use std::{
    sync::{Arc, RwLock},
    thread,
    time::{Duration, Instant},
};

use crossbeam_channel::{select, tick, unbounded};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, poll};
use log::{debug, error};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Margin},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{Block, Paragraph, Wrap},
};

use crate::{error::BodaResult, state, state::state::max_scroll, util};

const KEY_SEQUENCE_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug)]
pub struct Manager {
    action_tx: crossbeam_channel::Sender<state::action::Ui>,
    pending_g: Option<Instant>,
    content_height: u16,
}

impl Manager {
    pub fn new() -> (Manager, crossbeam_channel::Receiver<state::action::Ui>) {
        let (tx, rx) = unbounded::<state::action::Ui>();

        (
            Manager {
                action_tx: tx,
                pending_g: None,
                content_height: 0,
            },
            rx,
        )
    }
}

impl Manager {
    pub fn run(mut self, state: Arc<RwLock<state::state::State>>) -> thread::JoinHandle<()> {
        thread::spawn(move || {
            let mut terminal = setup_terminal();
            let ticker = tick(Duration::from_millis(16));

            loop {
                select! {
                    recv(ticker) -> _ => {
                        let state = state.read().unwrap();
                        if !state.global.running {
                            cleanup_terminal();
                            break;
                        }

                        terminal.draw(|frame| self.render(frame, &state)).unwrap();
                        while poll(Duration::from_secs(0)).unwrap() {
                            self.handle_crossterm_events().unwrap();
                        }
                    }
                }
            }
        })
    }

    fn handle_crossterm_events(&mut self) -> BodaResult<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
            _ => {}
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        debug!("key event");

        if key.modifiers.is_empty() && key.code == KeyCode::Char('g') {
            let now = Instant::now();
            if self
                .pending_g
                .is_some_and(|previous_g| now.duration_since(previous_g) <= KEY_SEQUENCE_TIMEOUT)
            {
                self.pending_g = None;
                self.action_tx.send(state::action::Ui::ScrollToTop).unwrap();
            } else {
                self.pending_g = Some(now);
            }
            return;
        }
        self.pending_g = None;

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
                self.action_tx
                    .send(state::action::Ui::ScrollDown(self.content_height))
                    .unwrap();
            }
            (_, KeyCode::Char('k')) => {
                self.action_tx
                    .send(state::action::Ui::ScrollUp(self.content_height))
                    .unwrap();
            }
            (_, KeyCode::Char('G')) => {
                self.action_tx
                    .send(state::action::Ui::ScrollToBottom(self.content_height))
                    .unwrap();
            }
            (KeyModifiers::CONTROL, KeyCode::Char('d') | KeyCode::Char('D')) => {
                self.action_tx
                    .send(state::action::Ui::ScrollHalfDown(self.content_height))
                    .unwrap();
            }
            (KeyModifiers::CONTROL, KeyCode::Char('u') | KeyCode::Char('U')) => {
                self.action_tx
                    .send(state::action::Ui::ScrollHalfUp(self.content_height))
                    .unwrap();
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
            (KeyModifiers::CONTROL, KeyCode::Char('p') | KeyCode::Char('P')) => {
                self.action_tx.send(state::action::Ui::SelectPrev).unwrap();
            }
            (KeyModifiers::CONTROL, KeyCode::Char('n') | KeyCode::Char('N')) => {
                self.action_tx.send(state::action::Ui::SelectNext).unwrap();
            }
            (_, KeyCode::Char('n')) => {
                self.action_tx
                    .send(state::action::Ui::ToggleLineNumbers)
                    .unwrap();
            }
            (_, KeyCode::Char('t')) => {
                self.action_tx
                    .send(state::action::Ui::ToggleTailFollow(self.content_height))
                    .unwrap();
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
gg: Scroll to top
G: Scroll to bottom
t: Toggle tail follow
k: Scroll Up
Ctrl+d: Half-page Down
Ctrl+u: Half-page Up
<Space>: Show History
Ctrl+p: Show previous
Ctrl+n: Show next
n: Toggle line numbers
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
        self.content_height = content_chunks[0].height;

        frame.render_widget(
            Paragraph::new(util::chrono::format_duration(state.global.interval)).block(
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
                Some(r) => format!("{}", r.start),
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
            let result_lines = result.get_content();
            let scroll_offset = output_scroll_offset(
                state.ui.follow_tail,
                result_lines.len(),
                self.content_height,
                state.ui.vertical_scroll,
            );
            let (content, style) = (
                Text::from(
                    result_lines
                        .iter()
                        .enumerate()
                        .map(|(index, line)| {
                            let mut spans = Vec::with_capacity(2);
                            if state.ui.show_line_numbers {
                                spans.push(Span::styled(
                                    format!("{:>4} ", index + 1),
                                    Style::default().fg(Color::DarkGray),
                                ));
                            }
                            spans.push(Span::raw(line.clone()));
                            Line::from(spans)
                        })
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
                    .wrap(Wrap { trim: false })
                    .scroll((scroll_offset, 0)),
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
                    let mut lines = vec![];

                    lines.push(Span::styled(
                        format!("{}", summary.start.time()),
                        if state.ui.target_command.is_target(summary) {
                            Style::default().bg(Color::DarkGray)
                        } else {
                            Style::default()
                        },
                    ));

                    lines.push(Span::raw(" "));
                    let status = match summary.status {
                        Some(0) => ("0".to_string(), Style::default().fg(Color::Green)),
                        Some(s) => (format!("{}", s), Style::default().fg(Color::Red)),
                        None => ("Running".to_string(), Style::default().fg(Color::Gray)),
                    };
                    lines.push(Span::styled(status.0, status.1));

                    if let Some(end) = summary.end {
                        lines.push(Span::raw(" "));
                        lines.push(Span::styled(
                            util::chrono::format_duration((end - summary.start).to_std().unwrap()),
                            Style::default().fg(Color::Gray),
                        ))
                    }

                    Line::from(lines)
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

fn output_scroll_offset(
    follow_tail: bool,
    content_length: usize,
    viewport_height: u16,
    current_offset: u16,
) -> u16 {
    if follow_tail {
        max_scroll(content_length, viewport_height)
    } else {
        current_offset
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

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::Manager;
    use crate::state::action;

    #[test]
    fn gg_scrolls_to_top() {
        let (mut manager, action_rx) = Manager::new();
        manager.on_key_event(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE));
        manager.on_key_event(KeyEvent::new(KeyCode::Char('g'), KeyModifiers::NONE));

        assert!(matches!(
            action_rx.try_recv().unwrap(),
            action::Ui::ScrollToTop
        ));
    }

    #[test]
    fn uppercase_g_scrolls_to_bottom() {
        let (mut manager, action_rx) = Manager::new();
        manager.content_height = 20;

        manager.on_key_event(KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT));

        assert!(matches!(
            action_rx.try_recv().unwrap(),
            action::Ui::ScrollToBottom(20)
        ));
    }

    #[test]
    fn ctrl_n_selects_next() {
        let (mut manager, action_rx) = Manager::new();
        manager.on_key_event(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::CONTROL));

        assert!(matches!(
            action_rx.try_recv().unwrap(),
            action::Ui::SelectNext
        ));
    }

    #[test]
    fn ctrl_p_selects_previous() {
        let (mut manager, action_rx) = Manager::new();
        manager.on_key_event(KeyEvent::new(KeyCode::Char('p'), KeyModifiers::CONTROL));

        assert!(matches!(
            action_rx.try_recv().unwrap(),
            action::Ui::SelectPrev
        ));
    }

    #[test]
    fn n_toggles_line_numbers() {
        let (mut manager, action_rx) = Manager::new();
        manager.on_key_event(KeyEvent::new(KeyCode::Char('n'), KeyModifiers::NONE));

        assert!(matches!(
            action_rx.try_recv().unwrap(),
            action::Ui::ToggleLineNumbers
        ));
    }

    #[test]
    fn t_toggles_tail_follow() {
        let (mut manager, action_rx) = Manager::new();
        manager.content_height = 24;
        manager.on_key_event(KeyEvent::new(KeyCode::Char('t'), KeyModifiers::NONE));

        assert!(matches!(
            action_rx.try_recv().unwrap(),
            action::Ui::ToggleTailFollow(24)
        ));
    }

    #[test]
    fn tail_follow_tracks_content_growth() {
        use super::output_scroll_offset;

        assert_eq!(output_scroll_offset(true, 100, 20, 5), 80);
        assert_eq!(output_scroll_offset(true, 150, 20, 5), 130);
        assert_eq!(output_scroll_offset(false, 150, 20, 5), 5);
    }
}
