use rat_nexus::{Component, Context, EventContext, Event, Action, Entity};
use crate::model::{AppState, LocalState};
use ratatui::{
    layout::{Layout, Constraint, Direction, Alignment},
    widgets::{Block, Borders, Paragraph, Gauge, List, ListItem, Sparkline, Chart, Axis, Dataset, GraphType},
    style::{Style, Color, Modifier, Stylize},
    symbols,
};
use crossterm::event::KeyCode;

pub struct CounterPage {
    title: &'static str,
    state: Entity<AppState>,
    local: Entity<LocalState>,
}

impl CounterPage {
    pub fn new(title: &'static str, state: Entity<AppState>, local: Entity<LocalState>) -> Self {
        Self { 
            title, 
            state, 
            local,
        }
    }

    fn log(&self, msg: String) {
        let _ = self.local.update(|s| {
            s.logs.push(msg);
            if s.logs.len() > 15 {
                s.logs.remove(0);
            }
        });
    }
}

impl Component for CounterPage {
    fn on_init(&mut self, cx: &mut Context<Self>) {
        let local = self.local.clone();
        let app = cx.app.clone();
        
        // Task 1: Clock
        cx.app.spawn(move |_| async move {
            loop {
                let now = chrono::Local::now().format("%H:%M:%S").to_string();
                let _ = local.update(|s| s.current_time = now);
                app.refresh();
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });

        // Task 2: Mock System Load
        let local_val = self.local.clone();
        let app2 = cx.app.clone();
        cx.app.spawn(move |_| async move {
            use rand::Rng;
            loop {
                {
                    let mut rng = rand::thread_rng();
                    let _ = local_val.update(|s| {
                        s.system_load.remove(0);
                        s.system_load.push(rng.gen_range(0..100));
                    });
                }
                app2.refresh();
                tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            }
        });
    }

    fn on_exit(&mut self, _cx: &mut Context<Self>) {
        self.log("Lifecycle: on_exit called (Leaving Page)".to_string());
    }

    fn on_shutdown(&mut self, _cx: &mut Context<Self>) {
        // Since we are shutting down, we can't really see this in the log UI, 
        // but we could perform cleanup here.
        eprintln!("Lifecycle: on_shutdown called for CounterPage");
    }

    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        cx.subscribe(&self.state);
        cx.subscribe(&self.local);

        let counter_state = self.state.read(|s| s.clone()).expect("failed to read global state");
        let local = self.local.read(|s| s.clone()).expect("failed to read local state");

        // Main Layout: Header, Main, Footer
        let area = cx.area;
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Main
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // --- Render Header ---
        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(20)])
            .split(main_layout[0]);

        let title = Paragraph::new(format!("Nexus Framework Demo - {}", self.title))
            .bold()
            .alignment(Alignment::Left)
            .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::Cyan)));
        frame.render_widget(title, header_chunks[0]);

        let clock = Paragraph::new(local.current_time)
            .cyan()
            .alignment(Alignment::Right)
            .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::Cyan)));
        frame.render_widget(clock, header_chunks[1]);

        // --- Render Main Area ---
        let body_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30), // Left: State & Controls
                Constraint::Percentage(40), // Center: Activity
                Constraint::Percentage(30), // Right: Inspector
            ])
            .split(main_layout[1]);

        // 1. GLOBAL STATE & CONTROLS
        let left_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(7),
                Constraint::Min(0),
            ])
            .split(body_layout[0]);

        let counter_block = Block::default().title(" Global State ").borders(Borders::ALL).border_type(ratatui::widgets::BorderType::Rounded);
        let counter_p = Paragraph::new(vec![
            format!("Counter Value: {}", counter_state.counter).into(),
            "".into(),
            "Press 'j' to Increment".into(),
            "Press 'k' to Decrement".into(),
        ]).alignment(Alignment::Center).block(counter_block);
        frame.render_widget(counter_p, left_chunks[0]);

        let controls_text = vec![
            " [L] Toggle Layout Mode ".into(),
            " [W] Start Async Worker ".into(),
            " [C] Clear Event Logs   ".into(),
            " [M] Return to Menu     ".into(),
            " [Q] Quit Application   ".into(),
        ];
        let controls_p = Paragraph::new(controls_text)
            .block(Block::default().title(" Framework Controls ").borders(Borders::ALL).border_type(ratatui::widgets::BorderType::Double));
        frame.render_widget(controls_p, left_chunks[1]);

        // 2. ACTIVITY (Center)
        let center_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // Sparkline
                Constraint::Length(3),  // Progress
                Constraint::Min(0),     // Chart
            ])
            .split(body_layout[1]);

        let sparkline = Sparkline::default()
            .block(Block::default().title(" Mock Net Activity ").borders(Borders::ALL))
            .data(&local.system_load)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(sparkline, center_chunks[0]);

        let gauge = Gauge::default()
            .block(Block::default().title(" Process Status ").borders(Borders::ALL))
            .gauge_style(Style::default().fg(if local.is_working { Color::Yellow } else { Color::Green }))
            .percent(local.progress)
            .use_unicode(true)
            .label(if local.is_working { "RUNNING" } else { "IDLE" });
        frame.render_widget(gauge, center_chunks[1]);

        // Counter History Chart
        let history_data: Vec<(f64, f64)> = counter_state.history.iter()
            .enumerate()
            .map(|(i, &v)| (i as f64, v as f64))
            .collect();

        let datasets = vec![
            Dataset::default()
                .name("Counter")
                .marker(symbols::Marker::Dot)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Cyan))
                .data(&history_data),
        ];

        let x_axis = Axis::default()
            .title("Time")
            .style(Style::default().fg(Color::Gray))
            .bounds([0.0, 50.0]);

        let max_val = counter_state.history.iter().max().copied().unwrap_or(10) as f64;
        let y_axis = Axis::default()
            .title("Value")
            .style(Style::default().fg(Color::Gray))
            .bounds([0.0, max_val + 10.0])
            .labels(vec![
                ratatui::text::Span::raw("0"),
                ratatui::text::Span::raw((max_val / 2.0).to_string()),
                ratatui::text::Span::raw(max_val.to_string()),
            ]);

        let chart = Chart::new(datasets)
            .block(Block::default().title(" Global Counter Synchronization ").borders(Borders::ALL))
            .x_axis(x_axis)
            .y_axis(y_axis);
        frame.render_widget(chart, center_chunks[2]);

        // 3. LOGS & INSPECTOR (Right)
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(60),
                Constraint::Percentage(40),
            ])
            .split(body_layout[2]);

        let items: Vec<ListItem> = local.logs.iter().rev().map(|l| ListItem::new(l.as_str())).collect();
        let logs_list = List::new(items)
            .block(Block::default().title(" Event Stream ").borders(Borders::ALL))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));
        frame.render_widget(logs_list, right_chunks[0]);

        let inspect_text = vec![
            format!("Area: {}x{}", cx.area.width, cx.area.height),
            format!("Origin: {}, {}", cx.area.x, cx.area.y),
            format!("Layout: {}", if local.layout_horizontal { "Horizontal" } else { "Vertical" }),
            format!("Entities: Subscribed to 2"),
        ];
        let inspector = Paragraph::new(inspect_text.join("\n"))
            .block(Block::default().title(" Context Inspector ").borders(Borders::ALL).fg(Color::DarkGray));
        frame.render_widget(inspector, right_chunks[1]);

        // --- Render Footer ---
        let footer = Paragraph::new("Nexus v1.0 | Built with Ratatui | Press 'q' to Quit")
            .style(Style::default().bg(Color::Blue).fg(Color::White))
            .alignment(Alignment::Center);
        frame.render_widget(footer, main_layout[2]);
    }

    fn handle_event(&mut self, event: Event, cx: &mut EventContext<Self>) -> Option<Action> {
        match event {
            Event::Mouse(mouse) => {
                self.log(format!("Mouse Event: {:?}", mouse.kind));
                use crossterm::event::MouseButton;
                match mouse.kind {
                    crossterm::event::MouseEventKind::Down(MouseButton::Left) => {
                        let _ = self.state.update(|s| {
                            s.counter += 1;
                            s.history.push(s.counter as u64);
                            if s.history.len() > 50 { s.history.remove(0); }
                        });
                        self.log("Mouse: Left Click -> Inc".to_string());
                        None
                    }
                    crossterm::event::MouseEventKind::Down(MouseButton::Right) => {
                        let _ = self.state.update(|s| {
                            s.counter -= 1;
                            s.history.push(s.counter as u64);
                            if s.history.len() > 50 { s.history.remove(0); }
                        });
                        self.log("Mouse: Right Click -> Dec".to_string());
                        None
                    }
                    crossterm::event::MouseEventKind::ScrollUp => {
                        let _ = self.state.update(|s| {
                            s.counter += 1;
                            s.history.push(s.counter as u64);
                            if s.history.len() > 50 { s.history.remove(0); }
                        });
                        self.log("Mouse: Scroll Up -> Inc".to_string());
                        None
                    }
                    crossterm::event::MouseEventKind::ScrollDown => {
                        let _ = self.state.update(|s| {
                            s.counter -= 1;
                            s.history.push(s.counter as u64);
                            if s.history.len() > 50 { s.history.remove(0); }
                        });
                        self.log("Mouse: Scroll Down -> Dec".to_string());
                        None
                    }
                    _ => None,
                }
            }
            Event::Key(key) if key.code == KeyCode::Char('l') => {
                let _ = self.local.update(|s| s.layout_horizontal = !s.layout_horizontal);
                self.log("Layout Toggled".to_string());
                None
            }
            Event::Key(key) if key.code == KeyCode::Char('w') => {
                 let local = self.local.clone();
                 let is_working = local.read(|s| s.is_working).unwrap_or(false);
                 if !is_working {
                    self.log("Worker Job Started".to_string());
                    let _ = local.update(|s| { s.is_working = true; s.progress = 0; });
                    
                    cx.app.spawn(move |_| async move {
                        for i in 1..=10 {
                            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                            let _ = local.update(|s| s.progress = i * 10);
                        }
                        let _ = local.update(|s| { 
                            s.is_working = false; 
                            s.logs.push("Worker Job: Complete".to_string());
                        });
                    });
                 }
                 None
            }
            Event::Key(key) if key.code == KeyCode::Char('c') => {
                let _ = self.local.update(|s| s.logs.clear());
                None
            }
            Event::Key(key) if key.code == KeyCode::Char('j') => {
                let _ = self.state.update(|s| {
                    s.counter += 1;
                    s.history.push(s.counter as u64);
                    if s.history.len() > 50 { s.history.remove(0); }
                });
                self.log(format!("Action: Inc -> {}", self.state.read(|s| s.counter).unwrap_or(0)));
                None
            }
            Event::Key(key) if key.code == KeyCode::Char('k') => {
                let _ = self.state.update(|s| {
                    s.counter -= 1;
                    s.history.push(s.counter as u64);
                    if s.history.len() > 50 { s.history.remove(0); }
                });
                self.log(format!("Action: Dec -> {}", self.state.read(|s| s.counter).unwrap_or(0)));
                None
            }
            Event::Key(key) if key.code == KeyCode::Char('m') => {
                Some(Action::Navigate("menu".to_string()))
            }
            Event::Key(key) if key.code == KeyCode::Char('q') => {
                Some(Action::Quit)
            }
            _ => None,
        }
    }
}
