//! Flappy Bird - Classic arcade game clone
//! Showcases: Real-time game loop, collision detection, Entity state, Componentization

use rat_nexus::{Component, Context, EventContext, Event, Action, Entity, TaskTracker};
use ratatui::{
    layout::{Layout, Constraint, Direction, Alignment},
    widgets::{Block, Borders, Paragraph, BorderType, canvas::{Canvas, Rectangle, Points, Context as CanvasContext}},
    style::{Style, Color, Modifier},
    text::Line,
};
use crossterm::event::KeyCode;

const GRAVITY: f64 = 0.22;
const JUMP_FORCE: f64 = 1.6;
const PIPE_GAP: f64 = 15.0;
const PIPE_WIDTH: f64 = 5.0;
const PIPE_SPEED: f64 = 0.8;

// ============================================
// Bird Component - Drawn with particles
// ============================================
#[derive(Clone)]
pub struct Bird {
    pub x: f64,
    pub y: f64,
    pub vy: f64,
    pub radius: f64,
    pub alive: bool,
}

impl Bird {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y, vy: 0.0, radius: 1.8, alive: true }
    }

    pub fn update(&mut self) {
        self.vy -= GRAVITY;
        self.y += self.vy;
    }

    pub fn flap(&mut self) {
        self.vy = JUMP_FORCE;
    }

    pub fn check_bounds(&mut self, ground: f64, ceiling: f64) {
        if self.y < ground + self.radius {
            self.y = ground + self.radius;
            self.alive = false;
        }
        if self.y > ceiling - self.radius {
            self.y = ceiling - self.radius;
            self.vy = 0.0;
        }
    }

    pub fn collides_with_pipe(&self, pipe_x: f64, gap_y: f64) -> bool {
        if self.x + self.radius > pipe_x && self.x - self.radius < pipe_x + PIPE_WIDTH {
            let in_gap = self.y > gap_y - PIPE_GAP / 2.0 + self.radius
                && self.y < gap_y + PIPE_GAP / 2.0 - self.radius;
            return !in_gap;
        }
        false
    }

    pub fn reset(&mut self, y: f64) {
        self.y = y;
        self.vy = 0.0;
        self.alive = true;
    }

    /// Render bird using emoji + particles (~64 particles for effects)
    pub fn render(&self, ctx: &mut CanvasContext) {
        let x = self.x;
        let y = self.y;

        // === Main body - Emoji 🐤 ===
        let bird_emoji = if self.alive { "🐤" } else { "💀" };
        ctx.print(x - 0.5, y, Line::styled(bird_emoji, Style::default()));

        // === Wing particles (~24) - flapping animation ===
        let wing_color = if self.alive { Color::Rgb(255, 200, 50) } else { Color::DarkGray };
        let wing_y_offset = if self.vy > 0.3 {
            1.0  // up
        } else if self.vy < -0.3 {
            -0.6 // down
        } else {
            0.2  // neutral
        };

        let mut wing_points: Vec<(f64, f64)> = vec![];
        for i in 0..8 {
            let t = i as f64 / 7.0;
            let wx = x - 1.0 - t * 1.2;
            let wy = y + wing_y_offset * (1.0 - t * 0.3);
            wing_points.push((wx, wy));
            wing_points.push((wx + 0.1, wy + 0.1));
            wing_points.push((wx - 0.1, wy - 0.1));
        }
        ctx.draw(&Points { coords: &wing_points, color: wing_color });

        // === Tail particles (~18) ===
        let tail_color = if self.alive { Color::Rgb(220, 160, 0) } else { Color::DarkGray };
        let mut tail_points: Vec<(f64, f64)> = vec![];
        for i in 0..6 {
            let spread = (i as f64 - 2.5) * 0.12;
            tail_points.push((x - 1.5, y + spread));
            tail_points.push((x - 1.7, y + spread * 1.3));
            tail_points.push((x - 1.9, y + spread * 1.5));
        }
        ctx.draw(&Points { coords: &tail_points, color: tail_color });

        // === Sparkle trail (~12) - movement effect ===
        if self.alive && self.vy.abs() > 0.2 {
            let sparkle_color = Color::Rgb(255, 255, 150);
            let mut sparkles: Vec<(f64, f64)> = vec![];
            for i in 0..4 {
                let offset = i as f64 * 0.5;
                sparkles.push((x - 2.0 - offset, y + (i as f64 * 0.1).sin() * 0.3));
                sparkles.push((x - 2.2 - offset, y - 0.2 + (i as f64 * 0.15).cos() * 0.2));
                sparkles.push((x - 2.1 - offset, y + 0.1));
            }
            ctx.draw(&Points { coords: &sparkles, color: sparkle_color });
        }

        // === Speed lines (~10) when moving fast ===
        if self.vy > 0.5 {
            // Going up - lines below
            let up_lines: Vec<(f64, f64)> = (0..10)
                .map(|i| (x - 0.5 + (i as f64 * 0.2), y - 1.5 - (i as f64 * 0.1)))
                .collect();
            ctx.draw(&Points { coords: &up_lines, color: Color::White });
        } else if self.vy < -0.5 {
            // Falling - lines above
            let down_lines: Vec<(f64, f64)> = (0..10)
                .map(|i| (x - 0.5 + (i as f64 * 0.2), y + 1.5 + (i as f64 * 0.1)))
                .collect();
            ctx.draw(&Points { coords: &down_lines, color: Color::Cyan });
        }
    }
}

// ============================================
// Pipe Component
// ============================================
#[derive(Clone)]
pub struct Pipe {
    pub x: f64,
    pub gap_y: f64,
    pub passed: bool,
}

impl Pipe {
    pub fn new(x: f64, gap_y: f64) -> Self {
        Self { x, gap_y, passed: false }
    }

    pub fn update(&mut self) {
        self.x -= PIPE_SPEED;
    }

    pub fn render(&self, ctx: &mut CanvasContext) {
        // Top pipe
        ctx.draw(&Rectangle {
            x: self.x,
            y: self.gap_y + PIPE_GAP / 2.0,
            width: PIPE_WIDTH,
            height: 50.0 - (self.gap_y + PIPE_GAP / 2.0),
            color: Color::Green,
        });
        // Bottom pipe
        ctx.draw(&Rectangle {
            x: self.x,
            y: 2.0,
            width: PIPE_WIDTH,
            height: (self.gap_y - PIPE_GAP / 2.0 - 2.0).max(0.0),
            color: Color::Green,
        });
        // Pipe caps
        ctx.draw(&Rectangle {
            x: self.x - 0.5,
            y: self.gap_y + PIPE_GAP / 2.0 - 1.0,
            width: PIPE_WIDTH + 1.0,
            height: 1.2,
            color: Color::LightGreen,
        });
        ctx.draw(&Rectangle {
            x: self.x - 0.5,
            y: self.gap_y - PIPE_GAP / 2.0,
            width: PIPE_WIDTH + 1.0,
            height: 1.2,
            color: Color::LightGreen,
        });
    }
}

// ============================================
// Game State
// ============================================
#[derive(Clone)]
pub struct FlappyState {
    bird: Bird,
    pipes: Vec<Pipe>,
    score: u32,
    high_score: u32,
    started: bool,
    tick: u64,
}

impl Default for FlappyState {
    fn default() -> Self {
        Self {
            bird: Bird::new(20.0, 25.0),
            pipes: vec![],
            score: 0,
            high_score: 0,
            started: false,
            tick: 0,
        }
    }
}

impl FlappyState {
    fn reset(&mut self) {
        if self.score > self.high_score {
            self.high_score = self.score;
        }
        self.bird.reset(25.0);
        self.pipes.clear();
        self.score = 0;
        self.started = false;
        self.tick = 0;
    }
}

pub struct FlappyPage {
    state: Option<Entity<FlappyState>>,
    tasks: TaskTracker,
}

impl Default for FlappyPage {
    fn default() -> Self {
        Self {
            state: None,
            tasks: TaskTracker::new(),
        }
    }
}

impl Component for FlappyPage {
    fn on_mount(&mut self, cx: &mut Context<Self>) {
        // Initialize state entity
        let state = cx.new_entity(FlappyState::default());
        self.state = Some(Entity::clone(&state));

        let handle = cx.spawn_detached_task(move |app| async move {
            use rand::Rng;
            use rand::SeedableRng;
            let mut rng = rand::rngs::StdRng::from_entropy();

            loop {
                let (started, alive) = state.read(|s| (s.started, s.bird.alive)).unwrap_or((false, false));

                if started && alive {
                    let _ = state.update(|s| {
                        s.tick += 1;

                        // Update bird
                        s.bird.update();
                        s.bird.check_bounds(2.0, 48.0);

                        // Spawn pipes
                        if s.tick % 55 == 0 {
                            let gap_y = rng.gen_range(14.0..36.0);
                            s.pipes.push(Pipe::new(105.0, gap_y));
                        }

                        // Update pipes
                        for pipe in s.pipes.iter_mut() {
                            pipe.update();

                            if !pipe.passed && pipe.x + PIPE_WIDTH < s.bird.x {
                                pipe.passed = true;
                                s.score += 1;
                            }

                            if s.bird.collides_with_pipe(pipe.x, pipe.gap_y) {
                                s.bird.alive = false;
                            }
                        }

                        s.pipes.retain(|p| p.x > -PIPE_WIDTH);
                    });
                    app.refresh();
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(33)).await;
            }
        });
        self.tasks.track(handle);
    }

    fn on_exit(&mut self, _cx: &mut Context<Self>) {
        self.tasks.abort_all();
    }

    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        if let Some(state) = &self.state {
            cx.subscribe(state);
            let state_data = state.read(|s| s.clone()).unwrap_or_default();
        let area = frame.area();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        // Header
        let status = if !state_data.bird.alive { "GAME OVER" } else if !state_data.started { "READY" } else { "FLYING" };
        let header_color = if !state_data.bird.alive { Color::Red } else { Color::Yellow };
        let header = Paragraph::new(format!(" Score: {}  |  Best: {}  |  {} ", state_data.score, state_data.high_score, status))
            .style(Style::default().fg(header_color).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
        frame.render_widget(header, layout[0]);

        // Game canvas
        let bird = state_data.bird.clone();
        let pipes = state_data.pipes.clone();
        let started = state_data.started;

        let canvas = Canvas::default()
            .block(Block::default()
                .title(" Flappy Bird ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Cyan)))
            .x_bounds([0.0, 100.0])
            .y_bounds([0.0, 50.0])
            .paint(move |ctx| {
                // Ground
                ctx.draw(&Rectangle { x: 0.0, y: 0.0, width: 100.0, height: 2.0, color: Color::DarkGray });

                // Render pipes
                for pipe in &pipes {
                    pipe.render(ctx);
                }

                // Render bird (particle-based)
                bird.render(ctx);

                // Clouds
                ctx.print(12.0, 44.0, Line::styled("☁", Style::default().fg(Color::White)));
                ctx.print(55.0, 46.0, Line::styled("☁", Style::default().fg(Color::White)));
                ctx.print(85.0, 42.0, Line::styled("☁", Style::default().fg(Color::White)));

                // Instructions
                if !started && bird.alive {
                    ctx.print(33.0, 28.0, Line::styled("Press SPACE to fly!", Style::default().fg(Color::White)));
                }
                if !bird.alive {
                    ctx.print(40.0, 28.0, Line::styled("R to restart", Style::default().fg(Color::White)));
                }
            });
        frame.render_widget(canvas, layout[1]);

        // Footer
        let footer_color = if !state_data.bird.alive { Color::Red } else { Color::Yellow };
        let footer = Paragraph::new(" SPACE Flap | R Reset | M Menu | Q Quit ")
            .style(Style::default().bg(footer_color).fg(Color::Black))
            .alignment(Alignment::Center);
        frame.render_widget(footer, layout[2]);
        }
    }

    fn handle_event(&mut self, event: Event, _cx: &mut EventContext<Self>) -> Option<Action> {
        if let Some(state) = &self.state {
        match event {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') => Some(Action::Quit),
                KeyCode::Char('m') | KeyCode::Esc => Some(Action::Navigate("menu".to_string())),
                KeyCode::Char('r') => {
                    let _ = state.update(|s| s.reset());
                    None
                }
                KeyCode::Char(' ') | KeyCode::Up => {
                    let _ = state.update(|s| {
                        if !s.bird.alive {
                            s.reset();
                        }
                        if !s.started {
                            s.started = true;
                        }
                        s.bird.flap();
                    });
                    None
                }
                _ => None,
            },
            _ => None,
        }
        } else {
            None
        }
    }
}
