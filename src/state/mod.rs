//! State management.
//!
//! Provides utilities for shared application state with reactivity.

use std::sync::{Arc, Mutex};
use tokio::sync::watch;

/// Shared state wrapper.
pub type SharedState<T> = Arc<Mutex<T>>;

/// Create a new shared state.
pub fn new_shared_state<T>(state: T) -> SharedState<T> {
    Arc::new(Mutex::new(state))
}

/// Entity wrapper for shared state, inspired by gpui.
///
/// Provides a convenient interface for reading and writing shared state,
/// plus reactivity via watch channel.
pub struct Entity<T>
where
    T: Clone + Send + Sync + 'static,
{
    inner: SharedState<T>,
    tx: watch::Sender<T>,
}

impl<T> Entity<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Create a new entity with the given initial value.
    pub fn new(value: T) -> Self {
        let (tx, _) = watch::channel(value.clone());
        Self {
            inner: new_shared_state(value),
            tx,
        }
    }

    /// Immutably borrow the inner value.
    pub fn get(&self) -> std::sync::MutexGuard<'_, T> {
        self.inner.lock().unwrap()
    }

    /// Mutably borrow the inner value.
    /// Note: changes made via this guard will NOT be automatically notified.
    /// Use `update` or `set` for notification.
    pub fn get_mut(&self) -> std::sync::MutexGuard<'_, T> {
        self.inner.lock().unwrap()
    }

    /// Replace the inner value and notify subscribers.
    pub fn set(&self, value: T) {
        *self.inner.lock().unwrap() = value.clone();
        let _ = self.tx.send(value);
    }

    /// Update the inner value using a closure and notify subscribers.
    pub fn update<F>(&self, f: F)
    where
        F: FnOnce(&mut T),
    {
        let mut guard = self.inner.lock().unwrap();
        f(&mut *guard);
        let new_value = (*guard).clone();
        drop(guard);
        let _ = self.tx.send(new_value);
    }

    /// Subscribe to changes of this entity.
    /// Returns a watch receiver that can be used to await changes.
    pub fn subscribe(&self) -> watch::Receiver<T> {
        self.tx.subscribe()
    }

    /// Get the current value via watch (non‑blocking).
    pub fn watch(&self) -> T {
        self.tx.borrow().clone()
    }
}

impl<T> Clone for Entity<T>
where
    T: Clone + Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            tx: self.tx.clone(),
        }
    }
}