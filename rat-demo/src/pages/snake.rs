use rat_nexus::{Component, Context, EventContext, Event, Action, Entity, AppContext, TaskTracker};
use ratatui::{
    layout::{Layout, Constraint, Direction, Alignment, Rect},
    widgets::{Block, Borders, Paragraph},
    style::{Style, Color, Stylize},
    text::Line,
};
use crossterm::event::KeyCode;
use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq)]
pub enum Direction2D {
    Up,
    Down,
    Left,
    Right,
}

impl Direction2D {
    fn opposite(&self) -> Self {
        match self {
            Direction2D::Up => Direction2D::Down,
            Direction2D::Down => Direction2D::Up,
            Direction2D::Left => Direction2D::Right,
            Direction2D::Right => Direction2D::Left,
        }
    }
}

#[derive(Clone)]
pub struct SnakeState {
    snake: VecDeque<(i32, i32)>,
    direction: Direction2D,
    food: (i32, i32),
    score: u32,
    game_over: bool,
    paused: bool,
    grid_width: i32,
    grid_height: i32,
    high_score: u32,
}

impl Default for SnakeState {
    fn default() -> Self {
        let grid_width = 40;
        let grid_height = 20;
        let mut snake = VecDeque::new();
        snake.push_back((grid_width / 2, grid_height / 2));
        snake.push_back((grid_width / 2 - 1, grid_height / 2));
        snake.push_back((grid_width / 2 - 2, grid_height / 2));

        Self {
            snake,
            direction: Direction2D::Right,
            food: (grid_width / 4 * 3, grid_height / 2),
            score: 0,
            game_over: false,
            paused: false,
            grid_width,
            grid_height,
            high_score: 0,
        }
    }
}

impl SnakeState {
    fn spawn_food(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        loop {
            let x = rng.gen_range(1..self.grid_width - 1);
            let y = rng.gen_range(1..self.grid_height - 1);
            if !self.snake.contains(&(x, y)) {
                self.food = (x, y);
                break;
            }
        }
    }

    fn reset(&mut self) {
        if self.score > self.high_score {
            self.high_score = self.score;
        }
        let grid_width = self.grid_width;
        let grid_height = self.grid_height;
        let high_score = self.high_score;
        *self = Self::default();
        self.grid_width = grid_width;
        self.grid_height = grid_height;
        self.high_score = high_score;
    }

    fn tick(&mut self) -> bool {
        if self.game_over || self.paused {
            return false;
        }

        let head = *self.snake.front().unwrap();
        let new_head = match self.direction {
            Direction2D::Up => (head.0, head.1 + 1),
            Direction2D::Down => (head.0, head.1 - 1),
            Direction2D::Left => (head.0 - 1, head.1),
            Direction2D::Right => (head.0 + 1, head.1),
        };

        // Check wall collision
        if new_head.0 <= 0 || new_head.0 >= self.grid_width - 1
            || new_head.1 <= 0 || new_head.1 >= self.grid_height - 1 {
            self.game_over = true;
            return true;
        }

        // Check self collision
        if self.snake.contains(&new_head) {
            self.game_over = true;
            return true;
        }

        self.snake.push_front(new_head);

        // Check food
        if new_head == self.food {
            self.score += 10;
            self.spawn_food();
        } else {
            self.snake.pop_back();
        }

        true
    }
}

pub struct SnakePage {
    state: Entity<SnakeState>,
    tasks: TaskTracker,
}

impl SnakePage {
    pub fn new(cx: &AppContext) -> Self {
        Self {
            state: cx.new_entity(SnakeState::default()),
            tasks: TaskTracker::new(),
        }
    }
}

impl Component for SnakePage {
    fn on_mount(&mut self, cx: &mut Context<Self>) {
        // on_mount is called once, no need for initialized flag
        let state = Entity::clone(&self.state);
        let app = AppContext::clone(&cx.app);

        // Game loop - tick every 100ms
        let handle = cx.spawn_task(move |_| async move {
            loop {
                {
                    let changed = state.update(|s| s.tick()).unwrap_or(false);
                    if changed {
                        app.refresh();
                    }
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        });
        self.tasks.track(handle);
    }

    fn on_exit(&mut self, _cx: &mut Context<Self>) {
        // Pause game and cancel tasks when leaving
        let _ = self.state.update(|s| s.paused = true);
        self.tasks.abort_all();
    }

    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        cx.subscribe(&self.state);
        let state = self.state.read(|s| s.clone()).unwrap_or_default();

        let area = frame.area();
        let main_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(area);

        // Header
        let header_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(20),
                Constraint::Length(20),
            ])
            .split(main_layout[0]);

        let title = Paragraph::new(" 🐍 SNAKE GAME ")
            .bold()
            .green()
            .alignment(Alignment::Left)
            .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::Green)));
        frame.render_widget(title, header_chunks[0]);

        let score_text = format!("Score: {}", state.score);
        let score = Paragraph::new(score_text)
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Right)
            .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::Green)));
        frame.render_widget(score, header_chunks[1]);

        let high_score_text = format!("High: {}", state.high_score);
        let high_score = Paragraph::new(high_score_text)
            .style(Style::default().fg(Color::Cyan))
            .alignment(Alignment::Right)
            .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(Color::Green)));
        frame.render_widget(high_score, header_chunks[2]);

        // Game area
        let game_block = Block::default()
            .title(if state.paused { " PAUSED " } else if state.game_over { " GAME OVER " } else { " Playing " })
            .borders(Borders::ALL)
            .border_style(Style::default().fg(if state.game_over { Color::Red } else if state.paused { Color::Yellow } else { Color::Green }));

        let game_area = game_block.inner(main_layout[1]);
        frame.render_widget(game_block, main_layout[1]);

        // Render game using text-based grid
        self.render_game(frame, game_area, &state);

        // Footer
        let footer_text = if state.game_over {
            " Press R to restart | M to return to menu | Q to quit "
        } else if state.paused {
            " Press SPACE to resume | M to return to menu | Q to quit "
        } else {
            " ←↑↓→ Move | SPACE Pause | M Menu | Q Quit "
        };
        let footer = Paragraph::new(footer_text)
            .style(Style::default().bg(Color::DarkGray).fg(Color::White))
            .alignment(Alignment::Center);
        frame.render_widget(footer, main_layout[2]);
    }

    fn handle_event(&mut self, event: Event, _cx: &mut EventContext<Self>) -> Option<Action> {
        let state = self.state.read(|s| s.clone()).unwrap_or_default();

        match event {
            Event::Key(key) if key.code == KeyCode::Char('q') => {
                return Some(Action::Quit);
            }
            Event::Key(key) if key.code == KeyCode::Char('m') => {
                return Some(Action::Navigate("menu".to_string()));
            }
            Event::Key(key) if key.code == KeyCode::Char('r') => {
                let _ = self.state.update(|s| s.reset());
                return None;
            }
            Event::Key(key) if key.code == KeyCode::Char(' ') => {
                if !state.game_over {
                    let _ = self.state.update(|s| s.paused = !s.paused);
                }
                return None;
            }
            _ => {}
        }

        if state.game_over || state.paused {
            return None;
        }

        match event {
            Event::Key(key) => {
                let new_dir = match key.code {
                    KeyCode::Up | KeyCode::Char('w') => Some(Direction2D::Up),
                    KeyCode::Down | KeyCode::Char('s') => Some(Direction2D::Down),
                    KeyCode::Left | KeyCode::Char('a') => Some(Direction2D::Left),
                    KeyCode::Right | KeyCode::Char('d') => Some(Direction2D::Right),
                    _ => None,
                };

                if let Some(dir) = new_dir {
                    let _ = self.state.update(|s| {
                        if dir != s.direction.opposite() {
                            s.direction = dir;
                        }
                    });
                }
                None
            }
            _ => None,
        }
    }
}

impl SnakePage {
    fn render_game(&self, frame: &mut ratatui::Frame, area: Rect, state: &SnakeState) {
        if area.width < 3 || area.height < 3 {
            return;
        }

        // Calculate cell size based on available area
        let cell_width = area.width as i32 / state.grid_width;
        let cell_height = area.height as i32 / state.grid_height;

        if cell_width < 1 || cell_height < 1 {
            // Fallback: render as simple text
            let mut lines: Vec<Line> = Vec::new();
            for y in (0..state.grid_height).rev() {
                let mut row = String::new();
                for x in 0..state.grid_width {
                    let pos = (x, y);
                    if x == 0 || x == state.grid_width - 1 || y == 0 || y == state.grid_height - 1 {
                        row.push('█');
                    } else if state.snake.front() == Some(&pos) {
                        row.push('◉');
                    } else if state.snake.contains(&pos) {
                        row.push('●');
                    } else if state.food == pos {
                        row.push('★');
                    } else {
                        row.push(' ');
                    }
                }
                lines.push(Line::from(row));
            }
            let para = Paragraph::new(lines);
            frame.render_widget(para, area);
            return;
        }

        // Build the game display
        let mut lines: Vec<Line> = Vec::new();

        let display_height = (area.height as i32).min(state.grid_height);
        let display_width = (area.width as i32 / 2).min(state.grid_width); // Each cell takes 2 chars

        for y in (0..display_height).rev() {
            let mut row = String::new();
            for x in 0..display_width {
                let pos = (x, y);
                if x == 0 || x == display_width - 1 || y == 0 || y == display_height - 1 {
                    row.push_str("██");
                } else if state.snake.front() == Some(&pos) {
                    row.push_str("◉ ");
                } else if state.snake.contains(&pos) {
                    row.push_str("● ");
                } else if state.food == pos {
                    row.push_str("★ ");
                } else {
                    row.push_str("  ");
                }
            }
            lines.push(Line::styled(row, Style::default().fg(
                if y == 0 || y == display_height - 1 { Color::DarkGray } else { Color::Green }
            )));
        }

        let para = Paragraph::new(lines).alignment(Alignment::Center);
        frame.render_widget(para, area);
    }
}
