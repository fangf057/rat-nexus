//! Component trait definition.

use ratatui::prelude::*;

/// Event type for component interactions.
#[derive(Debug, Clone)]
pub enum Event {
    Key(crossterm::event::KeyEvent),
    Mouse(crossterm::event::MouseEvent),
    Resize(u16, u16),
    Custom(String),
}

/// Action that a component can return after handling an event.
#[derive(Debug)]
pub enum Action {
    Navigate(String), // route
    Back,
    Quit,
    Noop,
}

/// The core Component trait.
///
/// Components are the building blocks of the TUI application.
/// They can render themselves and handle events.
pub trait Component: Send + Sync {
    /// Render the component into the given area.
    fn render(&self, f: &mut Frame, area: Rect);

    /// Handle an event, returning an optional action.
    fn handle_event(&mut self, event: Event) -> Option<Action> {
        let _ = event;
        None
    }
}