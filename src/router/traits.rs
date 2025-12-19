//! Router trait definition.

use crate::component::traits::AnyComponent;

/// Represents a route identifier.
pub type Route = String;

/// The Router trait manages navigation between components.
pub trait Router {
    /// Navigate to a given route.
    fn navigate(&mut self, route: Route);

    /// Get the current route.
    fn current_route(&self) -> &Route;

    /// Get a mutable reference to the current component.
    fn current_component(&mut self) -> &mut dyn AnyComponent;
}