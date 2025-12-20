use rat_nexus::{Component, Context, EventContext, Event, Action, Route, Entity};
use ratatui::widgets::Paragraph;
use crossterm::event::KeyCode;
use crate::model::AppState;

pub struct Menu {
    selected: usize,
    options: Vec<(&'static str, Route)>,
    state: Entity<AppState>,
}

impl Menu {
    pub fn new(state: Entity<AppState>) -> Self {
        Self {
            selected: 0,
            options: vec![
                ("Counter Page A", "page_a".to_string()),
                ("Counter Page B", "page_b".to_string()),
                ("Snake Game 🐍", "snake".to_string()),
                ("Exit", "exit".to_string()),
            ],
            state,
        }
    }
}

impl Component for Menu {
    fn on_mount(&mut self, _cx: &mut Context<Self>) {
        // Menu initialized (called once)
    }

    fn on_enter(&mut self, _cx: &mut Context<Self>) {
        // Menu entered (called each navigation)
    }

    fn on_exit(&mut self, _cx: &mut Context<Self>) {
        // Leaving menu
    }

    fn on_shutdown(&mut self, _cx: &mut Context<Self>) {
        eprintln!("Lifecycle: on_shutdown called for Menu");
    }

    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        use ratatui::layout::{Layout, Constraint, Direction, Alignment};
        use ratatui::widgets::{Block, Borders, List, ListItem, BorderType};
        use ratatui::style::{Style, Modifier, Color, Stylize};

        cx.subscribe(&self.state);
        let counter = self.state.read(|s| s.counter).unwrap_or(0);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .margin(2)
            .split(frame.area());

        let header = Paragraph::new(" NEXUS SYSTEM MENU ")
            .bold()
            .cyan()
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Double));
        frame.render_widget(header, chunks[0]);

        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60),
                Constraint::Percentage(40),
            ])
            .split(chunks[1]);

        let items: Vec<ListItem> = self.options.iter()
            .enumerate()
            .map(|(i, (label, _))| {
                if i == self.selected {
                    ListItem::new(format!(" > {} ", label))
                        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
                } else {
                    ListItem::new(format!("   {} ", label))
                        .style(Style::default().fg(Color::Gray))
                }
            })
            .collect();

        let list = List::new(items)
            .block(Block::default()
                .title(" Select Module ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray)));
        
        frame.render_widget(list, body_chunks[0]);

        let info = Paragraph::new(vec![
            " Global Status ".bold().cyan().into(),
            "".into(),
            format!(" System Counter: {}", counter).into(),
            "".into(),
            " Framework: Active ".green().into(),
            " Reactivity: Enabled ".green().into(),
        ])
        .block(Block::default().title(" Monitor ").borders(Borders::ALL).border_type(BorderType::Rounded));
        frame.render_widget(info, body_chunks[1]);

        let footer = Paragraph::new(" Use ↑↓ to navigate, ENTER to select ")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        frame.render_widget(footer, chunks[2]);
    }

    fn handle_event(&mut self, event: Event, _cx: &mut EventContext<Self>) -> Option<Action> {
        match event {
            Event::Key(key) if key.code == KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                None
            }
            Event::Key(key) if key.code == KeyCode::Down => {
                if self.selected < self.options.len() - 1 {
                    self.selected += 1;
                }
                None
            }
            Event::Key(key) if key.code == KeyCode::Enter => {
                let (_, route) = &self.options[self.selected];
                if route == "exit" {
                    Some(Action::Quit)
                } else {
                    Some(Action::Navigate(route.clone()))
                }
            }
            Event::Key(key) if key.code == KeyCode::Char('q') => {
                Some(Action::Quit)
            }
            _ => None,
        }
    }
}
