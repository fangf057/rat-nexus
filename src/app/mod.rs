//! Application struct and main loop.

use crate::component::traits::{Event, Action};
use crate::router::Router;
use ratatui::prelude::*;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, stdout};
use std::time::Duration;

/// The main application.
pub struct App<R>
where
    R: Router,
{
    router: R,
}

impl<R> App<R>
where
    R: Router,
{
    /// Create a new application with the given router.
    pub fn new(router: R) -> Self {
        Self { router }
    }

    /// Run the application loop.
    pub fn run(&mut self) -> io::Result<()> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let result = self.run_app_loop(&mut terminal);

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

    fn run_app_loop(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
        loop {
            terminal.draw(|frame| {
                let component = self.router.current_component();
                component.render(frame, frame.area());
            })?;

            if event::poll(Duration::from_millis(250))? {
                if let CrosstermEvent::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        let component = self.router.current_component();
                        let event = Event::Key(key);
                        if let Some(action) = component.handle_event(event) {
                            match action {
                                Action::Navigate(route) => self.router.navigate(route),
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