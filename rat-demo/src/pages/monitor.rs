//! System Monitor - Real-time system metrics visualization.
//!
//! Demonstrates:
//! - Real-time async data updates with TaskTracker
//! - Chart widget for time series
//! - Multiple Sparklines
//! - Table with dynamic data
//! - Complex layout composition

use rat_nexus::{Component, Context, EventContext, Event, Action, Entity, TaskTracker};
use crate::model::{AppState, MonitorState};
use ratatui::{
    layout::{Layout, Constraint, Direction, Alignment, Rect},
    widgets::{
        Block, Borders, Paragraph, Table, Row, Cell, Sparkline,
        BorderType, Chart, Axis, Dataset, GraphType,
    },
    style::{Style, Color, Modifier},
    text::{Line, Span},
    symbols,
};
use crossterm::event::KeyCode;

pub struct MonitorPage {
    app_state: Option<Entity<AppState>>,
    state: Option<Entity<MonitorState>>,
    tasks: TaskTracker,
}

impl Default for MonitorPage {
    fn default() -> Self {
        Self {
            app_state: None,
            state: None,
            tasks: TaskTracker::new(),
        }
    }
}

impl Component for MonitorPage {
    fn on_mount(&mut self, cx: &mut Context<Self>) {
        // Get or initialize shared AppState
        let app_state = cx.get_or_insert_with::<Entity<AppState>, _>(|| {
            cx.new_entity(AppState::default())
        }).expect("Failed to initialize AppState");
        self.app_state = Some(app_state);

        // Initialize MonitorState
        let state = cx.new_entity(MonitorState::default());
        self.state = Some(Entity::clone(&state));

        // Spawn data simulation task
        let handle = cx.spawn_detached_task(move |app| async move {
            use rand::Rng;
            use rand::SeedableRng;
            let mut rng = rand::rngs::StdRng::from_entropy();

            loop {
                let _ = state.update(|s| {
                    // Simulate CPU usage
                    s.cpu_history.remove(0);
                    s.cpu_history.push(rng.gen_range(20..80));

                    // Simulate memory usage
                    s.memory_history.remove(0);
                    let last_mem = *s.memory_history.last().unwrap_or(&50);
                    let delta: i64 = rng.gen_range(-5..6);
                    s.memory_history.push(((last_mem as i64 + delta).clamp(30, 70)) as u64);

                    // Simulate network
                    s.network_in.remove(0);
                    s.network_in.push(rng.gen_range(10..100));
                    s.network_out.remove(0);
                    s.network_out.push(rng.gen_range(5..50));

                    // Simulate CPU cores
                    for core in s.cpu_cores.iter_mut() {
                        *core = rng.gen_range(10..100);
                    }

                    // Update disk (slow change)
                    if rng.gen_bool(0.1) {
                        let delta: i16 = rng.gen_range(-2..3);
                        s.disk_usage = (s.disk_usage as i16 + delta).clamp(20, 80) as u16;
                    }

                    // Update processes
                    for proc in s.processes.iter_mut() {
                        proc.cpu = (proc.cpu + rng.gen_range(-0.5..0.5)).clamp(0.0, 10.0);
                        proc.memory = (proc.memory + rng.gen_range(-0.2..0.2)).clamp(0.1, 5.0);
                    }

                    // Uptime
                    s.uptime_secs += 1;
                });

                app.refresh();
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
        });
        self.tasks.track(handle);
    }

    fn on_exit(&mut self, _cx: &mut Context<Self>) {
        self.tasks.abort_all();
    }

    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        if let (Some(state), Some(app_state)) = (&self.state, &self.app_state) {
            cx.subscribe(state);
            cx.subscribe(app_state);

            let state_data = state.read(|s| s.clone()).unwrap_or_default();
            let app = app_state.read(|s| s.clone()).unwrap_or_default();
            let theme_color = app.theme.color();

        let area = frame.area();

        // Main layout: Header, Body, Footer
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Body
                Constraint::Length(3), // Footer
            ])
            .split(area);

        // Header with system info
        let uptime_str = format_uptime(state_data.uptime_secs);
        let header_text = format!(
            " 📊 System Monitor │ Uptime: {} │ Theme: {} ",
            uptime_str,
            app.theme.name()
        );
        let header = Paragraph::new(header_text)
            .style(Style::default().fg(theme_color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Left)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme_color)));
        frame.render_widget(header, main_layout[0]);

        // Body layout
        let body_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(65), Constraint::Percentage(35)])
            .split(main_layout[1]);

        // Left side: Charts
        self.render_charts(frame, body_layout[0], &state_data, theme_color);

        // Right side: Metrics and processes
        self.render_sidebar(frame, body_layout[1], &state_data, theme_color);

        // Footer
        let footer = Paragraph::new(" R Reset │ T Theme │ M Menu │ Q Quit │ Mouse: Scroll to adjust ")
            .style(Style::default().bg(theme_color).fg(Color::Black))
            .alignment(Alignment::Center);
        frame.render_widget(footer, main_layout[2]);
        }
    }

    fn handle_event(&mut self, event: Event, _cx: &mut EventContext<Self>) -> Option<Action> {
        if let (Some(state), Some(app_state)) = (&self.state, &self.app_state) {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => Some(Action::Quit),
                KeyCode::Char('m') | KeyCode::Esc => Some(Action::Navigate("menu".to_string())),
                KeyCode::Char('t') => {
                    let _ = app_state.update(|s| s.theme = s.theme.next());
                    None
                }
                KeyCode::Char('r') => {
                    // Reset all metrics
                    let _ = state.update(|s| {
                        s.cpu_history = vec![50; 60];
                        s.memory_history = vec![50; 60];
                        s.network_in = vec![50; 30];
                        s.network_out = vec![25; 30];
                        s.uptime_secs = 0;
                    });
                    None
                }
                _ => None,
            },
            Event::Mouse(mouse) => {
                use crossterm::event::{MouseEventKind, MouseButton};
                match mouse.kind {
                    MouseEventKind::ScrollUp => {
                        // Increase disk usage on scroll up
                        let _ = state.update(|s| {
                            s.disk_usage = (s.disk_usage + 5).min(100);
                        });
                        None
                    }
                    MouseEventKind::ScrollDown => {
                        // Decrease disk usage on scroll down
                        let _ = state.update(|s| {
                            s.disk_usage = s.disk_usage.saturating_sub(5);
                        });
                        None
                    }
                    MouseEventKind::Down(MouseButton::Left) => {
                        // Left click cycles theme
                        let _ = app_state.update(|s| s.theme = s.theme.next());
                        None
                    }
                    MouseEventKind::Down(MouseButton::Right) => {
                        // Right click resets
                        let _ = state.update(|s| {
                            s.uptime_secs = 0;
                        });
                        None
                    }
                    _ => None,
                }
            }
            _ => None,
        }
        } else {
            None
        }
    }
}

impl MonitorPage {
    fn render_charts(&self, frame: &mut ratatui::Frame, area: Rect, state: &MonitorState, theme_color: Color) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // CPU/Memory chart
                Constraint::Percentage(25), // Network sparklines
                Constraint::Percentage(25), // CPU cores
            ])
            .margin(1)
            .split(area);

        // CPU & Memory Chart
        let cpu_data: Vec<(f64, f64)> = state.cpu_history.iter()
            .enumerate()
            .map(|(i, &v)| (i as f64, v as f64))
            .collect();

        let mem_data: Vec<(f64, f64)> = state.memory_history.iter()
            .enumerate()
            .map(|(i, &v)| (i as f64, v as f64))
            .collect();

        let datasets = vec![
            Dataset::default()
                .name("CPU")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(theme_color))
                .data(&cpu_data),
            Dataset::default()
                .name("Memory")
                .marker(symbols::Marker::Braille)
                .graph_type(GraphType::Line)
                .style(Style::default().fg(Color::Green))
                .data(&mem_data),
        ];

        let chart = Chart::new(datasets)
            .block(Block::default()
                .title(" CPU & Memory Usage (%) ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme_color)))
            .x_axis(Axis::default()
                .title("Time")
                .style(Style::default().fg(Color::DarkGray))
                .bounds([0.0, 60.0]))
            .y_axis(Axis::default()
                .title("Usage %")
                .style(Style::default().fg(Color::DarkGray))
                .bounds([0.0, 100.0])
                .labels(["0", "50", "100"]));

        frame.render_widget(chart, chunks[0]);

        // Network sparklines
        let net_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(chunks[1]);

        let net_in_spark = Sparkline::default()
            .block(Block::default()
                .title(" ↓ Network In (KB/s) ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Green)))
            .data(&state.network_in)
            .style(Style::default().fg(Color::Green));
        frame.render_widget(net_in_spark, net_chunks[0]);

        let net_out_spark = Sparkline::default()
            .block(Block::default()
                .title(" ↑ Network Out (KB/s) ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Yellow)))
            .data(&state.network_out)
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(net_out_spark, net_chunks[1]);

        // CPU cores as mini gauges
        let core_block = Block::default()
            .title(" CPU Cores ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme_color));

        let inner = core_block.inner(chunks[2]);
        frame.render_widget(core_block, chunks[2]);

        let core_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Ratio(1, state.cpu_cores.len() as u32); state.cpu_cores.len()])
            .split(inner);

        for (i, (chunk, &usage)) in core_chunks.iter().zip(state.cpu_cores.iter()).enumerate() {
            let color = if usage > 80 { Color::Red } else if usage > 50 { Color::Yellow } else { Color::Green };
            let _label = format!("C{}", i);

            // Create a vertical gauge effect using text
            let height = chunk.height as u16;
            let filled = (usage as u16 * height / 100).min(height);

            let mut lines = Vec::new();
            for h in (0..height).rev() {
                let c = if h < filled { "█" } else { " " };
                lines.push(Line::styled(c, Style::default().fg(color)));
            }

            let gauge_para = Paragraph::new(lines).alignment(Alignment::Center);
            frame.render_widget(gauge_para, *chunk);
        }
    }

    fn render_sidebar(&self, frame: &mut ratatui::Frame, area: Rect, state: &MonitorState, theme_color: Color) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(8),  // Disk & quick stats
                Constraint::Min(0),     // Process table
            ])
            .margin(1)
            .split(area);

        // Quick stats
        let avg_cpu = state.cpu_history.iter().sum::<u64>() / state.cpu_history.len().max(1) as u64;
        let avg_mem = state.memory_history.iter().sum::<u64>() / state.memory_history.len().max(1) as u64;
        let net_in_total: u64 = state.network_in.iter().sum();
        let net_out_total: u64 = state.network_out.iter().sum();

        let stats_text = vec![
            Line::from(vec![
                Span::styled("CPU Avg:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:>3}%", avg_cpu), Style::default().fg(theme_color)),
            ]),
            Line::from(vec![
                Span::styled("Mem Avg:  ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:>3}%", avg_mem), Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("Disk:     ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:>3}%", state.disk_usage), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("Net ↓:    ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:>5} KB", net_in_total), Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("Net ↑:    ", Style::default().fg(Color::DarkGray)),
                Span::styled(format!("{:>5} KB", net_out_total), Style::default().fg(Color::Yellow)),
            ]),
        ];

        let stats = Paragraph::new(stats_text)
            .block(Block::default()
                .title(" Quick Stats ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(theme_color)));
        frame.render_widget(stats, chunks[0]);

        // Process table
        let rows: Vec<Row> = state.processes.iter()
            .map(|p| {
                let cpu_color = if p.cpu > 5.0 { Color::Red } else if p.cpu > 2.0 { Color::Yellow } else { Color::Green };
                Row::new(vec![
                    Cell::from(format!("{}", p.pid)).style(Style::default().fg(Color::DarkGray)),
                    Cell::from(p.name.clone()),
                    Cell::from(format!("{:.1}%", p.cpu)).style(Style::default().fg(cpu_color)),
                    Cell::from(format!("{:.1}%", p.memory)),
                ])
            })
            .collect();

        let table = Table::new(
            rows,
            [Constraint::Length(6), Constraint::Min(10), Constraint::Length(6), Constraint::Length(6)],
        )
        .header(
            Row::new(vec!["PID", "Name", "CPU", "Mem"])
                .style(Style::default().fg(theme_color).add_modifier(Modifier::BOLD))
                .bottom_margin(1),
        )
        .block(Block::default()
            .title(" Processes ")
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(theme_color)));

        frame.render_widget(table, chunks[1]);
    }
}

fn format_uptime(secs: u64) -> String {
    let hours = secs / 3600;
    let mins = (secs % 3600) / 60;
    let secs = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}
