//! Particles Demo - Animated particle system
//! Showcases: spawn_task, Entity updates, real-time animation, TaskTracker

use rat_nexus::{Component, Context, EventContext, Event, Action, Entity, TaskTracker};
use ratatui::{
    layout::{Layout, Constraint, Direction, Alignment},
    widgets::{Block, Borders, Paragraph, BorderType, canvas::{Canvas, Points}},
    style::{Style, Color},
};
use crossterm::event::KeyCode;

#[derive(Clone)]
pub struct Particle {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
    life: u8,
    color: Color,
}

#[derive(Clone, Default)]
pub struct ParticlesState {
    particles: Vec<Particle>,
    spawn_x: f64,
    spawn_y: f64,
    paused: bool,
    total_spawned: u64,
}

pub struct ParticlesPage {
    state: Entity<ParticlesState>,
    tasks: TaskTracker,
}

impl ParticlesPage {
    pub fn new(cx: &rat_nexus::AppContext) -> Self {
        Self {
            state: cx.new_entity(ParticlesState { spawn_x: 50.0, spawn_y: 25.0, ..Default::default() }),
            tasks: TaskTracker::new(),
        }
    }
}

impl Component for ParticlesPage {
    fn on_mount(&mut self, cx: &mut Context<Self>) {
        let state = Entity::clone(&self.state);

        // Particle physics update loop
        let handle = cx.spawn_detached_task(move |app| async move {
            use rand::Rng;
            use rand::SeedableRng;
            let mut rng = rand::rngs::StdRng::from_entropy();

            loop {
                let paused = state.read(|s| s.paused).unwrap_or(false);
                if !paused {
                    let _ = state.update(|s| {
                        // Spawn new particles
                        for _ in 0..3 {
                            let angle = rng.gen_range(0.0..std::f64::consts::TAU);
                            let speed = rng.gen_range(0.5..2.0);
                            s.particles.push(Particle {
                                x: s.spawn_x,
                                y: s.spawn_y,
                                vx: angle.cos() * speed,
                                vy: angle.sin() * speed,
                                life: rng.gen_range(40..80),
                                color: match rng.gen_range(0..5) {
                                    0 => Color::Red,
                                    1 => Color::Yellow,
                                    2 => Color::Green,
                                    3 => Color::Cyan,
                                    _ => Color::Magenta,
                                },
                            });
                            s.total_spawned += 1;
                        }

                        // Update particles
                        for p in s.particles.iter_mut() {
                            p.x += p.vx;
                            p.y += p.vy;
                            p.vy -= 0.03; // gravity
                            p.life = p.life.saturating_sub(1);
                        }

                        // Remove dead particles
                        s.particles.retain(|p| p.life > 0);
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
        cx.subscribe(&self.state);
        let state = self.state.read(|s| s.clone()).unwrap_or_default();
        let area = frame.area();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(3)])
            .split(area);

        // Header
        let status = if state.paused { "PAUSED" } else { "RUNNING" };
        let header = Paragraph::new(format!(
            " Particles: {}  |  Spawned: {}  |  {} ",
            state.particles.len(), state.total_spawned, status
        ))
        .style(Style::default().fg(Color::Cyan))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded));
        frame.render_widget(header, layout[0]);

        // Canvas
        let canvas_area = layout[1];
        let particles_data: Vec<_> = state.particles.iter()
            .map(|p| (p.x, p.y, p.color))
            .collect();

        let canvas = Canvas::default()
            .block(Block::default()
                .title(" Particle Fountain ")
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Magenta)))
            .x_bounds([0.0, 100.0])
            .y_bounds([0.0, 50.0])
            .paint(move |ctx| {
                for (x, y, color) in &particles_data {
                    ctx.draw(&Points {
                        coords: &[(*x, *y)],
                        color: *color,
                    });
                }
            });
        frame.render_widget(canvas, canvas_area);

        // Footer
        let color = if state.paused { Color::Yellow } else { Color::Magenta };
        let footer = Paragraph::new(" SPACE Pause | Arrow Keys Move | R Reset | M Menu | Q Quit ")
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
                    let _ = self.state.update(|s| s.paused = !s.paused);
                    None
                }
                KeyCode::Char('r') => {
                    let _ = self.state.update(|s| {
                        s.particles.clear();
                        s.total_spawned = 0;
                        s.spawn_x = 50.0;
                        s.spawn_y = 25.0;
                    });
                    None
                }
                KeyCode::Left => {
                    let _ = self.state.update(|s| s.spawn_x = (s.spawn_x - 5.0).max(5.0));
                    None
                }
                KeyCode::Right => {
                    let _ = self.state.update(|s| s.spawn_x = (s.spawn_x + 5.0).min(95.0));
                    None
                }
                KeyCode::Up => {
                    let _ = self.state.update(|s| s.spawn_y = (s.spawn_y + 3.0).min(45.0));
                    None
                }
                KeyCode::Down => {
                    let _ = self.state.update(|s| s.spawn_y = (s.spawn_y - 3.0).max(5.0));
                    None
                }
                _ => None,
            },
            _ => None,
        }
    }
}
