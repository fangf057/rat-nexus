//! High‑level Application abstraction inspired by GPUI.

use crate::component::Component;
use crate::component::traits::{Event, Action};
use crate::state::Entity;
use ratatui::prelude::*;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, stdout};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Application context providing access to global services.
pub struct Context {
    /// The root component to render, if set by the user.
    root: Arc<Mutex<Option<Arc<Mutex<dyn Component>>>>>,
}

impl Context {
    /// Create a new entity with the given value.
    pub fn new_entity<T>(&self, value: T) -> Entity<T> {
        Entity::new(value)
    }

    /// Schedule a task to be executed later (placeholder).
    pub fn spawn<F>(&self, _future: F)
    where
        F: std::future::Future<Output = ()> + 'static,
    {
        // In a real implementation, this would schedule the future on an async runtime.
        // For simplicity, we do nothing.
    }

    /// Set the root component of the application.
    pub fn set_root(&self, root: Arc<Mutex<dyn Component>>) {
        *self.root.lock().unwrap() = Some(root);
    }
}

/// Main application handle.
pub struct Application;

impl Application {
    /// Create a new application instance.
    pub fn new() -> Self {
        Self
    }

    /// Run the application with the given closure that receives a context.
    pub fn run<F>(self, setup: F) -> io::Result<()>
    where
        F: FnOnce(&Context) -> io::Result<()>,
    {
        // Create a placeholder for the root component.
        let root = Arc::new(Mutex::new(None));
        let context = Context { root: root.clone() };

        // Allow the user to set up their components via the context.
        setup(&context)?;

        // Determine the actual root component (user may have set it).
        let actual_root = {
            let guard = root.lock().unwrap();
            guard.clone().unwrap_or_else(|| Arc::new(Mutex::new(DummyComponent)))
        };

        // Run the main loop.
        self.run_loop(actual_root)
    }

    fn run_loop(&self, root: Arc<Mutex<dyn Component>>) -> io::Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app_loop(&mut terminal, root);

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        result
    }

    fn run_app_loop(
        &self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        root: Arc<Mutex<dyn Component>>,
    ) -> io::Result<()> {
        loop {
            terminal.draw(|frame| {
                let component = root.lock().unwrap();
                component.render(frame, frame.area());
            })?;

            if event::poll(Duration::from_millis(250))? {
                if let CrosstermEvent::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        let mut component = root.lock().unwrap();
                        let event = Event::Key(key);
                        if let Some(action) = component.handle_event(event) {
                            match action {
                                Action::Navigate(_) => {
                                    // In this simple example, navigation is not supported.
                                }
                                Action::Back => {
                                    // Back navigation is handled by the component itself.
                                }
                                Action::Quit => break,
                                Action::Noop => (),
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// Dummy component used as a fallback root.
struct DummyComponent;

impl Component for DummyComponent {
    fn render(&self, f: &mut Frame, area: Rect) {
        let paragraph = ratatui::widgets::Paragraph::new("No component set")
            .alignment(ratatui::layout::Alignment::Center);
        f.render_widget(paragraph, area);
    }

    fn handle_event(&mut self, _event: Event) -> Option<Action> {
        None
    }
}