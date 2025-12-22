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

/// Define an application with automatic routing and component dispatch.
///
/// This macro generates a Root component that automatically handles:
/// - RootRoute enum definition
/// - Root struct with router and all page fields
/// - Root::new(cx) with automatic page construction via Default trait
/// - Complete Component implementation with routing and lifecycle dispatch
/// - Navigation action handling
///
/// All components are created with Default::default() and can be customized
/// in their on_mount() lifecycle method.
///
/// Minimal syntax - just list the routes and page types!
///
/// # Example
/// ```ignore
/// use rat_nexus::define_app;
/// use crate::pages::{Menu, MonitorPage, TimerPage};
///
/// define_app! {
///     Menu => menu: Menu,
///     Monitor => monitor: MonitorPage,
///     Timer => timer: TimerPage,
/// }
///
/// // Automatically creates:
/// // - `enum RootRoute { Menu, Monitor, Timer }`
/// // - `pub struct Root { router, menu, monitor, timer }`
/// // - `impl Root { fn new() -> Self }`
/// // - `impl Component for Root` with full routing
///
/// // In main.rs:
/// // let root = Root::new();
/// ```
#[macro_export]
macro_rules! define_app {
    // Syntax 1: Simple - just routes, first route is default
    (
        $(
            $route:ident => $field:ident : $page:ty
        ),* $(,)?
    ) => {
        define_app!(@impl (Menu) $($route => $field : $page),*);
    };

    // Syntax 2: Full - with #[Root(default=...)] attribute
    (
        #[Root(default=$default_route:ident)]
        pub struct Root {
            $(
                $route:ident => $field:ident : $page:ty
            ),* $(,)?
        }
    ) => {
        define_app!(@impl ($default_route) $($route => $field : $page),*);
    };

    // Internal: actual implementation - takes default route and routes
    (@impl ($default_route:ident) $($route:ident => $field:ident : $page:ty),*) => {
        $crate::paste::paste! {
            use $crate::Component;
            // Generate RootRoute enum
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            pub enum RootRoute {
                $($route),*
            }

            impl Default for RootRoute {
                fn default() -> Self {
                    RootRoute::$default_route
                }
            }

            impl std::fmt::Display for RootRoute {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        $(Self::$route => write!(f, stringify!($route))),*
                    }
                }
            }

            /// Type-safe route parsing from strings.
            /// Returns error with available routes on mismatch.
            impl std::str::FromStr for RootRoute {
                type Err = String;

                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    let lower = s.to_lowercase();
                    $(
                        if lower == stringify!($route).to_lowercase() {
                            return Ok(RootRoute::$route);
                        }
                    )*
                    Err(format!(
                        "Unknown route: '{}'. Available routes: {}",
                        s,
                        vec![$(stringify!($route)),*].join(", ")
                    ))
                }
            }

            // Generate Root struct
            pub struct Root {
                router: $crate::Router<RootRoute>,
                $($field: $page),*
            }

            impl Root {
                /// Create a new Root instance.
                /// All pages are constructed using Default::default().
                /// Customize components in their on_mount() lifecycle method.
                pub fn new() -> Self {
                    Self {
                        router: $crate::Router::new(RootRoute::default()),
                        $($field: <$page>::default()),*
                    }
                }

                /// Get the current route
                pub fn current_route(&self) -> &RootRoute {
                    self.router.current()
                }

                /// Navigate to a route
                pub fn navigate(&mut self, route: RootRoute) {
                    self.router.navigate(route);
                }

                /// Go back to previous route
                pub fn go_back(&mut self) -> bool {
                    self.router.go_back()
                }

                /// Helper: Call on_enter for the given route
                fn call_on_enter(&mut self, route: RootRoute, cx: &mut $crate::Context<Self>) {
                    match route {
                        $(RootRoute::$route => self.$field.on_enter(&mut cx.cast())),*
                    }
                }

                /// Helper: Call on_exit for the given route
                fn call_on_exit(&mut self, route: RootRoute, cx: &mut $crate::Context<Self>) {
                    match route {
                        $(RootRoute::$route => self.$field.on_exit(&mut cx.cast())),*
                    }
                }
            }

            impl $crate::Component for Root {
                fn on_mount(&mut self, cx: &mut $crate::Context<Self>) {
                    $(self.$field.on_mount(&mut cx.cast());)*
                }

                fn on_enter(&mut self, cx: &mut $crate::Context<Self>) {
                    self.call_on_enter(*self.router.current(), cx);
                }

                fn on_exit(&mut self, cx: &mut $crate::Context<Self>) {
                    self.call_on_exit(*self.router.current(), cx);
                }

                fn on_shutdown(&mut self, cx: &mut $crate::Context<Self>) {
                    $(self.$field.on_shutdown(&mut cx.cast());)*
                }

                fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut $crate::Context<Self>) {
                    match self.router.current() {
                        $(RootRoute::$route => self.$field.render(frame, &mut cx.cast())),*
                    }
                }

                fn handle_event(&mut self, event: $crate::Event, cx: &mut $crate::EventContext<Self>) -> Option<$crate::Action> {
                    let current = *self.router.current();
                    let action = match current {
                        $(RootRoute::$route => self.$field.handle_event(event, &mut cx.cast())),*
                    };

                    // Handle navigation actions with type-safe routing
                    if let Some(action) = action {
                        match &action {
                            $crate::Action::Navigate(route_str) => {
                                // Type-safe route parsing with clear error messages
                                match route_str.parse::<RootRoute>() {
                                    Ok(target_route) => {
                                        // Exit current, enter new
                                        self.call_on_exit(current, cx);
                                        self.router.navigate(target_route);
                                        self.call_on_enter(target_route, cx);
                                    }
                                    Err(e) => {
                                        eprintln!("Navigation error: {}", e);
                                    }
                                }
                                None
                            }
                            $crate::Action::Back => {
                                // Exit current
                                self.call_on_exit(current, cx);

                                if self.router.go_back() {
                                    // Enter previous
                                    self.call_on_enter(*self.router.current(), cx);
                                }
                                None
                            }
                            $crate::Action::Quit => Some($crate::Action::Quit),
                            $crate::Action::Noop => None,
                        }
                    } else {
                        None
                    }
                }
            }
        }
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
