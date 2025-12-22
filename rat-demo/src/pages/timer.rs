//! Timer Demo - Stopwatch with lap times
//! Showcases: Entity state, spawn_task, TaskTracker, async updates

use rat_nexus::{Component, Context, EventContext, Event, Action, Entity, TaskTracker};
use ratatui::{
    layout::{Layout, Constraint, Direction, Alignment},
    widgets::{Block, Borders, Paragraph, List, ListItem, BorderType},
    style::{Style, Color, Modifier},
    text::{Line, Span},
};
use crossterm::event::KeyCode;

#[derive(Clone, Default)]
pub struct TimerState {
    pub elapsed_ms: u64,
    pub running: bool,
    pub laps: Vec<u64>,
}

pub struct TimerPage {
    state: Entity<TimerState>,
    tasks: TaskTracker,
}

impl TimerPage {
    pub fn new(cx: &rat_nexus::AppContext) -> Self {
        Self {
            state: cx.new_entity(TimerState::default()),
            tasks: TaskTracker::new(),
        }
    }
}

impl Component for TimerPage {
    fn on_mount(&mut self, cx: &mut Context<Self>) {
        let state = Entity::clone(&self.state);

        let handle = cx.spawn_detached_task(move |app| async move {
            loop {
                let running = state.read(|s| s.running).unwrap_or(false);
                if running {
                    let _ = state.update(|s| s.elapsed_ms += 10);
                    app.refresh();
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            }
        });
        self.tasks.track(handle);
    }

    fn on_exit(&mut self, _cx: &mut Context<Self>) {
        self.tasks.abort_all();
    }

    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        cx.subscribe(&self.state);
        let state = self.state.read(|s| s.clone()).unwrap_or_default();
        let area = frame.area();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(9), Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        // Timer display
        let time = format_time(state.elapsed_ms);
        let color = if state.running { Color::Green } else { Color::Yellow };

        let timer_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(&time, Style::default().fg(color).add_modifier(Modifier::BOLD)),
            ]).alignment(Alignment::Center),
            Line::from(""),
            Line::from(vec![
                Span::styled(
                    if state.running { "  RUNNING  " } else { "  STOPPED  " },
                    Style::default().fg(Color::Black).bg(color)
                ),
            ]).alignment(Alignment::Center),
        ];

        let timer = Paragraph::new(timer_lines)
            .block(Block::default()
                .title(" Stopwatch ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(color)));
        frame.render_widget(timer, layout[0]);

        // Lap times
        let lap_items: Vec<ListItem> = state.laps.iter().enumerate().rev()
            .map(|(i, &ms)| {
                ListItem::new(format!("  Lap {:02}  {}  ", i + 1, format_time(ms)))
                    .style(Style::default().fg(Color::Cyan))
            })
            .collect();

        let laps = List::new(lap_items)
            .block(Block::default()
                .title(format!(" Laps ({}) ", state.laps.len()))
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)));
        frame.render_widget(laps, layout[1]);

        // Footer
        let footer = Paragraph::new(" SPACE Start/Stop | L Lap | R Reset | M Menu | Q Quit ")
            .style(Style::default().bg(color).fg(Color::Black))
            .alignment(Alignment::Center);
        frame.render_widget(footer, layout[2]);
    }

    fn handle_event(&mut self, event: Event, _cx: &mut EventContext<Self>) -> Option<Action> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => Some(Action::Quit),
                KeyCode::Char('m') | KeyCode::Esc => Some(Action::Navigate("menu".to_string())),
                KeyCode::Char(' ') => {
                    let _ = self.state.update(|s| s.running = !s.running);
                    None
                }
                KeyCode::Char('l') => {
                    let _ = self.state.update(|s| {
                        if s.running || s.elapsed_ms > 0 {
                            s.laps.push(s.elapsed_ms);
                        }
                    });
                    None
                }
                KeyCode::Char('r') => {
                    let _ = self.state.update(|s| {
                        s.elapsed_ms = 0;
                        s.running = false;
                        s.laps.clear();
                    });
                    None
                }
                _ => None,
            },
            _ => None,
        }
    }
}

fn format_time(ms: u64) -> String {
    let mins = ms / 60000;
    let secs = (ms % 60000) / 1000;
    let centis = (ms % 1000) / 10;
    format!("{:02}:{:02}.{:02}", mins, secs, centis)
}
