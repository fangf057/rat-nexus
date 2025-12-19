use crate::application::{Context, EventContext};
use std::any::Any;

/// Event type for component interactions.
#[derive(Debug, Clone)]
pub enum Event {
    Key(crossterm::event::KeyEvent),
    Mouse(crossterm::event::MouseEvent),
    Resize(u16, u16),
    FocusGained,
    FocusLost,
    Paste(String),
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
    /// Called once when the component is first initialized or set as root.
    fn on_init(&mut self, cx: &mut Context<Self>) {
        let _ = cx;
    }

    /// Called when the component is removed from the active view (e.g. navigation).
    fn on_exit(&mut self, cx: &mut Context<Self>) {
        let _ = cx;
    }

    /// Called when the application is about to shut down.
    fn on_shutdown(&mut self, cx: &mut Context<Self>) {
        let _ = cx;
    }

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
pub trait AnyComponent: Any + Send + Sync + 'static {
    fn on_init_any(&mut self, cx: &mut Context<dyn AnyComponent>);
    fn on_exit_any(&mut self, cx: &mut Context<dyn AnyComponent>);
    fn on_shutdown_any(&mut self, cx: &mut Context<dyn AnyComponent>);
    fn render_any(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<dyn AnyComponent>);
    fn handle_event_any(&mut self, event: Event, cx: &mut EventContext<dyn AnyComponent>) -> Option<Action>;
}

impl<T: Component> AnyComponent for T {
    fn on_init_any(&mut self, cx: &mut Context<dyn AnyComponent>) {
        let mut cx = cx.cast::<Self>();
        self.on_init(&mut cx);
    }

    fn on_exit_any(&mut self, cx: &mut Context<dyn AnyComponent>) {
        let mut cx = cx.cast::<Self>();
        self.on_exit(&mut cx);
    }

    fn on_shutdown_any(&mut self, cx: &mut Context<dyn AnyComponent>) {
        let mut cx = cx.cast::<Self>();
        self.on_shutdown(&mut cx);
    }

    fn render_any(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<dyn AnyComponent>) {
        let mut cx = cx.cast::<Self>();
        self.render(frame, &mut cx);
    }

    fn handle_event_any(&mut self, event: Event, cx: &mut EventContext<dyn AnyComponent>) -> Option<Action> {
        let mut cx = cx.cast::<Self>();
        self.handle_event(event, &mut cx)
    }
}