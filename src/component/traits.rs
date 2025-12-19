//! Component trait definition.

use crate::application::{Context, EventContext};

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

/// The core Component trait for implementers.
pub trait Component: Send + Sync + 'static {
    /// Render the component into the given area.
    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>);

    /// Handle an event, returning an optional action.
    fn handle_event(&mut self, event: Event, cx: &mut EventContext<Self>) -> Option<Action> {
        let _ = event;
        let _ = cx;
        None
    }
}

/// A dyn-compatible version of the Component trait.
pub trait AnyComponent: Send + Sync + 'static {
    fn render_any(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<dyn AnyComponent>);
    fn handle_event_any(&mut self, event: Event, cx: &mut EventContext<dyn AnyComponent>) -> Option<Action>;
}

impl<T: Component> AnyComponent for T {
    fn render_any(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<dyn AnyComponent>) {
        let mut cx = cx.cast::<Self>();
        self.render(frame, &mut cx);
    }

    fn handle_event_any(&mut self, event: Event, cx: &mut EventContext<dyn AnyComponent>) -> Option<Action> {
        let cx = cx.cast::<Self>();
        self.handle_event(event, &mut {cx})
    }
}