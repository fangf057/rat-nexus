use rat_nexus::{Component, Context, EventContext, Event, Action, Route, Entity, Page, AppContext};
use ratatui::widgets::Paragraph;
use crossterm::event::KeyCode;
use crate::model::AppState;

pub struct Menu {
    selected: usize,
    options: Vec<(&'static str, &'static str, Route)>,  // (label, description, route)
    state: Entity<AppState>,
}

impl Page for Menu {
    fn build(cx: &AppContext) -> Self {
        // Get or create shared state with custom initialization
        let state = cx.get_or_insert_with::<Entity<AppState>, _>(|| {
            cx.new_entity(AppState::default())
        }).expect("Failed to initialize AppState");

        Self {
            selected: 0,
            options: vec![
                ("System Monitor", "Real-time charts, sparklines & metrics", "monitor".to_string()),
                ("Stopwatch", "Timer with laps & async updates", "timer".to_string()),
                ("Particles", "Animated particle fountain", "particles".to_string()),
                ("Flappy Bird", "Classic arcade game clone", "flappy".to_string()),
                ("Gomoku", "五子棋 Human vs AI", "tictactoe".to_string()),
                ("Exit", "Quit application", "exit".to_string()),
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
        // Cleanup
    }

    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        use ratatui::layout::{Layout, Constraint, Direction, Alignment};
        use ratatui::widgets::{Block, Borders, List, ListItem, BorderType};
        use ratatui::style::{Style, Modifier, Color};
        use ratatui::text::{Line, Span};

        cx.subscribe(&self.state);
        let app_state = self.state.read(|s| s.clone()).unwrap_or_default();
        let theme_color = app_state.theme.color();

        let area = frame.area();

        // Main layout
        let main_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // ASCII Art header
                Constraint::Min(0),     // Body
                Constraint::Length(3),  // Footer
            ])
            .split(area);

        // ASCII Art Header
        let ascii_art = vec![
            Line::from(""),
            Line::styled("  ██████╗  █████╗ ████████╗    ███╗   ██╗███████╗██╗  ██╗██╗   ██╗███████╗", Style::default().fg(theme_color)),
            Line::styled("  ██╔══██╗██╔══██╗╚══██╔══╝    ████╗  ██║██╔════╝╚██╗██╔╝██║   ██║██╔════╝", Style::default().fg(theme_color)),
            Line::styled("  ██████╔╝███████║   ██║       ██╔██╗ ██║█████╗   ╚███╔╝ ██║   ██║███████╗", Style::default().fg(theme_color)),
            Line::styled("  ██╔══██╗██╔══██║   ██║       ██║╚██╗██║██╔══╝   ██╔██╗ ██║   ██║╚════██║", Style::default().fg(theme_color)),
            Line::styled("  ██║  ██║██║  ██║   ██║       ██║ ╚████║███████╗██╔╝ ╚██╗╚██████╔╝███████║", Style::default().fg(theme_color)),
            Line::styled("  ╚═╝  ╚═╝╚═╝  ╚═╝   ╚═╝       ╚═╝  ╚═══╝╚══════╝╚═╝   ╚═╝ ╚═════╝ ╚══════╝", Style::default().fg(theme_color)),
        ];
        let header = Paragraph::new(ascii_art).alignment(Alignment::Center);
        frame.render_widget(header, main_chunks[0]);

        // Body: Menu + Info panel
        let body_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .margin(1)
            .split(main_chunks[1]);

        // Menu list with descriptions
        let items: Vec<ListItem> = self.options.iter()
            .enumerate()
            .map(|(i, (label, desc, _))| {
                let is_selected = i == self.selected;
                let prefix = if is_selected { "▶ " } else { "  " };

                let lines = vec![
                    Line::from(vec![
                        Span::styled(prefix, Style::default().fg(if is_selected { theme_color } else { Color::DarkGray })),
                        Span::styled(*label, Style::default()
                            .fg(if is_selected { theme_color } else { Color::White })
                            .add_modifier(if is_selected { Modifier::BOLD } else { Modifier::empty() })),
                    ]),
                    Line::from(vec![
                        Span::raw("    "),
                        Span::styled(*desc, Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
                    ]),
                ];
                ListItem::new(lines)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default()
                .title(" Select Module ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme_color)));

        frame.render_widget(list, body_chunks[0]);

        // Info panel with framework features
        let info_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled(" 🦀 ", Style::default()),
                Span::styled("Rat-Nexus Framework", Style::default().fg(theme_color).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::styled(" A reactive TUI framework inspired by GPUI", Style::default().fg(Color::DarkGray)),
            Line::from(""),
            Line::from(vec![
                Span::styled(" ✓ ", Style::default().fg(Color::Green)),
                Span::raw("Entity<T> reactive state"),
            ]),
            Line::from(vec![
                Span::styled(" ✓ ", Style::default().fg(Color::Green)),
                Span::raw("GPUI-style Context binding"),
            ]),
            Line::from(vec![
                Span::styled(" ✓ ", Style::default().fg(Color::Green)),
                Span::raw("Component lifecycle hooks"),
            ]),
            Line::from(vec![
                Span::styled(" ✓ ", Style::default().fg(Color::Green)),
                Span::raw("TaskTracker async management"),
            ]),
            Line::from(vec![
                Span::styled(" ✓ ", Style::default().fg(Color::Green)),
                Span::raw("Type-safe routing"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled(" Counter: ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{}", app_state.counter), Style::default().fg(theme_color)),
            ]),
            Line::from(vec![
                Span::styled(" Theme: ", Style::default().fg(Color::DarkGray)),
                Span::styled(app_state.theme.name(), Style::default().fg(theme_color)),
            ]),
        ];

        let info = Paragraph::new(info_lines)
            .block(Block::default()
                .title(" About ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme_color)));
        frame.render_widget(info, body_chunks[1]);

        // Footer
        let footer = Paragraph::new(" ↑/↓ Navigate │ Enter Select │ T Theme │ Q Quit ")
            .style(Style::default().bg(theme_color).fg(Color::Black))
            .alignment(Alignment::Center);
        frame.render_widget(footer, main_chunks[2]);
    }

    fn handle_event(&mut self, event: Event, _cx: &mut EventContext<Self>) -> Option<Action> {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.selected > 0 {
                        self.selected -= 1;
                    } else {
                        self.selected = self.options.len() - 1;
                    }
                    None
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.selected < self.options.len() - 1 {
                        self.selected += 1;
                    } else {
                        self.selected = 0;
                    }
                    None
                }
                KeyCode::Enter => {
                    let (_, _, route) = &self.options[self.selected];
                    if route == "exit" {
                        Some(Action::Quit)
                    } else {
                        Some(Action::Navigate(route.clone()))
                    }
                }
                KeyCode::Char('t') => {
                    let _ = self.state.update(|s| s.theme = s.theme.next());
                    None
                }
                KeyCode::Char('q') => Some(Action::Quit),
                _ => None,
            },
            _ => None,
        }
    }
}
