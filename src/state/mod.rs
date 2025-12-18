//! State management.
//!
//! Provides utilities for shared application state.

use std::sync::{Arc, Mutex};

/// Shared state wrapper.
pub type SharedState<T> = Arc<Mutex<T>>;

/// Create a new shared state.
pub fn new_shared_state<T>(state: T) -> SharedState<T> {
    Arc::new(Mutex::new(state))
}

/// Entity wrapper for shared state, inspired by gpui.
///
/// Provides a convenient interface for reading and writing shared state.
pub struct Entity<T>(SharedState<T>);

impl<T> Entity<T> {
    /// Create a new entity with the given initial value.
    pub fn new(value: T) -> Self {
        Self(new_shared_state(value))
    }

    /// Immutably borrow the inner value.
    pub fn get(&self) -> std::sync::MutexGuard<'_, T> {
        self.0.lock().unwrap()
    }

    /// Mutably borrow the inner value.
    pub fn get_mut(&self) -> std::sync::MutexGuard<'_, T> {
        self.0.lock().unwrap()
    }

    /// Replace the inner value.
    pub fn set(&self, value: T) {
        *self.get_mut() = value;
    }
}

impl<T> Clone for Entity<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}