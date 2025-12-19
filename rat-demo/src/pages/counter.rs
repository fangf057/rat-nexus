use rat_nexus::{Component, Context, EventContext, Event, Action, Entity};
use crate::model::{AppState, LocalState};
use ratatui::layout::{Layout, Constraint, Direction};
use ratatui::widgets::{Block, Borders, Paragraph, Gauge, List, ListItem};
use ratatui::style::{Style, Color};
use crossterm::event::KeyCode;

pub struct CounterPage {
    title: &'static str,
    state: Entity<AppState>,
    local: Entity<LocalState>,
}

impl CounterPage {
    pub fn new(title: &'static str, state: Entity<AppState>, local: Entity<LocalState>) -> Self {
        Self { title, state, local }
    }

    fn log(&self, msg: String) {
        self.local.update(|s| {
            s.logs.push(msg);
            if s.logs.len() > 10 {
                s.logs.remove(0);
            }
        }).expect("failed to log");
    }
}

impl Component for CounterPage {
    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        // Subscribe to updates
        cx.subscribe(&self.state);
        cx.subscribe(&self.local);

        let counter = self.state.read(|s| s.counter).expect("failed to read global state");
        let local = self.local.read(|s| s.clone()).expect("failed to read local state");

        let direction = if local.layout_horizontal { Direction::Horizontal } else { Direction::Vertical };
        
        let chunks = Layout::default()
            .direction(direction)
            .margin(1)
            .constraints([
                Constraint::Percentage(25), // Counter
                Constraint::Percentage(25), // Context Info
                Constraint::Percentage(25), // Progress
                Constraint::Percentage(25), // Logs
            ])
            .split(cx.area);

        // 1. Shared State (Counter)
        let text = format!("{}\nCounter: {}", self.title, counter);
        let p1 = Paragraph::new(text)
            .block(Block::default().title("1. Global State").borders(Borders::ALL))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(p1, chunks[0]);

        // 2. Context Info
        let area_info = format!(
            "Area: {}x{}\nOrigin: ({}, {})\n\nMode: {}", 
            cx.area.width, cx.area.height, cx.area.x, cx.area.y,
            if local.layout_horizontal { "Horizontal" } else { "Vertical" }
        );
        let p2 = Paragraph::new(area_info)
            .block(Block::default().title("2. Render Context").borders(Borders::ALL));
        frame.render_widget(p2, chunks[1]);

        // 3. Async Progress
        let gauge = Gauge::default()
            .block(Block::default().title("3. Async Task").borders(Borders::ALL))
            .gauge_style(Style::default().fg(if local.is_working { Color::Yellow } else { Color::Green }))
            .percent(local.progress);
        frame.render_widget(gauge, chunks[2]);

        // 4. Logs
        let items: Vec<ListItem> = local.logs.iter()
            .rev()
            .map(|l| ListItem::new(l.as_str()))
            .collect();
        let list = List::new(items)
            .block(Block::default().title("4. Event Log").borders(Borders::ALL));
        frame.render_widget(list, chunks[3]);
    }

    fn handle_event(&mut self, event: Event, cx: &mut EventContext<Self>) -> Option<Action> {
        match event {
            Event::Key(key) if key.code == KeyCode::Char('l') => {
                self.local.update(|s| s.layout_horizontal = !s.layout_horizontal).expect("failed to toggle layout");
                self.log("Layout Toggled".to_string());
                None
            }
            Event::Key(key) if key.code == KeyCode::Char('w') => {
                 let local = self.local.clone();
                 // If not already working
                 let is_working = local.read(|s| s.is_working).expect("failed to read work status");
                 if !is_working {
                    self.log("Async Task Started".to_string());
                    local.update(|s| { s.is_working = true; s.progress = 0; }).expect("failed to start task");
                    
                    cx.app.spawn(move |_| async move {
                        for i in 1..=10 {
                            tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                            local.update(|s| s.progress = i * 10).expect("failed to update progress");
                        }
                        local.update(|s| { s.is_working = false; s.logs.push("Task Complete".to_string()); }).expect("failed to complete task");
                    });
                 }
                 None
            }
             Event::Key(key) if key.code == KeyCode::Char('c') => {
                self.local.update(|s| s.logs.clear()).expect("failed to clear logs");
                None
            }
            Event::Key(key) if key.code == KeyCode::Char('j') => {
                self.state.update(|s| s.counter += 1).expect("failed to increment counter");
                self.log("Counter ++".to_string());
                None
            }
            Event::Key(key) if key.code == KeyCode::Char('k') => {
                self.state.update(|s| s.counter -= 1).expect("failed to decrement counter");
                self.log("Counter --".to_string());
                None
            }
            Event::Key(key) if key.code == KeyCode::Char('m') => {
                Some(Action::Navigate("menu".to_string()))
            }
            Event::Key(key) if key.code == KeyCode::Esc => {
                Some(Action::Back)
            }
            Event::Key(key) if key.code == KeyCode::Char('q') => {
                Some(Action::Quit)
            }
            _ => None,
        }
    }
}
