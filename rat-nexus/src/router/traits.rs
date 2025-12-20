//! Routing utilities for component navigation.
//!
//! Provides `Router` for managing navigation history and the `define_routes!` macro
//! for type-safe route definitions.

/// Legacy type alias for backward compatibility.
pub type Route = String;

/// A router that manages navigation history.
///
/// # Example
/// ```ignore
/// use rat_nexus::Router;
///
/// enum Route { Menu, Settings, Game }
///
/// let mut router = Router::new(Route::Menu);
/// router.navigate(Route::Settings);
/// assert_eq!(router.current(), &Route::Settings);
/// router.go_back();
/// assert_eq!(router.current(), &Route::Menu);
/// ```
#[derive(Debug, Clone)]
pub struct Router<R: Clone + PartialEq> {
    current: R,
    history: Vec<R>,
}

impl<R: Clone + PartialEq> Router<R> {
    /// Create a new router with the initial route.
    pub fn new(initial: R) -> Self {
        Self {
            current: initial,
            history: Vec::new(),
        }
    }

    /// Get the current route.
    pub fn current(&self) -> &R {
        &self.current
    }

    /// Navigate to a new route. The current route is pushed to history.
    pub fn navigate(&mut self, route: R) {
        if self.current != route {
            self.history.push(self.current.clone());
            self.current = route;
        }
    }

    /// Go back to the previous route. Returns true if successful.
    pub fn go_back(&mut self) -> bool {
        if let Some(prev) = self.history.pop() {
            self.current = prev;
            true
        } else {
            false
        }
    }

    /// Check if there's history to go back to.
    pub fn can_go_back(&self) -> bool {
        !self.history.is_empty()
    }

    /// Get the history length.
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Clear the navigation history.
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
}

/// Define a type-safe route enum with Display implementation.
///
/// # Example
/// ```ignore
/// use rat_nexus::define_routes;
///
/// define_routes! {
///     Menu,
///     Settings,
///     Game,
/// }
///
/// let route = Route::Menu;
/// assert_eq!(format!("{}", route), "Menu");
/// ```
#[macro_export]
macro_rules! define_routes {
    ($($name:ident),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Route {
            $($name),*
        }

        impl std::fmt::Display for Route {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Route::$name => write!(f, stringify!($name))),*
                }
            }
        }

        impl Default for Route {
            fn default() -> Self {
                // Default to the first variant
                define_routes!(@first $($name),*)
            }
        }
    };

    // Helper to get the first variant
    (@first $first:ident $(, $rest:ident)*) => {
        Route::$first
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum TestRoute {
        Home,
        Settings,
        Profile,
    }

    #[test]
    fn test_router_navigation() {
        let mut router = Router::new(TestRoute::Home);

        assert_eq!(router.current(), &TestRoute::Home);
        assert!(!router.can_go_back());

        router.navigate(TestRoute::Settings);
        assert_eq!(router.current(), &TestRoute::Settings);
        assert!(router.can_go_back());

        router.navigate(TestRoute::Profile);
        assert_eq!(router.current(), &TestRoute::Profile);
        assert_eq!(router.history_len(), 2);

        assert!(router.go_back());
        assert_eq!(router.current(), &TestRoute::Settings);

        assert!(router.go_back());
        assert_eq!(router.current(), &TestRoute::Home);

        assert!(!router.go_back());
        assert_eq!(router.current(), &TestRoute::Home);
    }

    #[test]
    fn test_router_no_duplicate_navigation() {
        let mut router = Router::new(TestRoute::Home);
        router.navigate(TestRoute::Home); // Same route
        assert_eq!(router.history_len(), 0); // No history added
    }
}
