use std::sync::{Arc, Mutex, Weak};
use tokio::sync::watch;

/// Shared state wrapper.
pub type SharedState<T> = Arc<Mutex<T>>;

/// Entity handle, inspired by gpui.
pub struct Entity<T: ?Sized + Send + Sync> {
    pub(crate) inner: SharedState<T>,
    tx: watch::Sender<()>,
}

/// A weak handle to an entity.
pub struct WeakEntity<T: ?Sized + Send + Sync> {
    pub(crate) inner: Weak<Mutex<T>>,
    tx: watch::Sender<()>,
}

impl<T: ?Sized + Send + Sync> Entity<T> {
    /// Update the inner value using a closure and notify subscribers.
    pub fn update<F, R>(&self, f: F) -> crate::Result<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.inner.lock().map_err(|_| crate::Error::LockPoisoned)?;
        let res = f(&mut *guard);
        drop(guard);
        let _ = self.tx.send(());
        Ok(res)
    }

    /// Read the inner value using a closure.
    pub fn read<F, R>(&self, f: F) -> crate::Result<R>
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.inner.lock().map_err(|_| crate::Error::LockPoisoned)?;
        Ok(f(&*guard))
    }

    /// Downcast this entity to a weak handle.
    pub fn downgrade(&self) -> WeakEntity<T> {
        WeakEntity {
            inner: Arc::downgrade(&self.inner),
            tx: self.tx.clone(),
        }
    }

    /// Subscribe to changes of this entity.
    pub fn subscribe(&self) -> watch::Receiver<()> {
        self.tx.subscribe()
    }
}

impl<T: ?Sized + Send + Sync> WeakEntity<T> {
    /// Upgrade this weak handle to a strong handle, if the entity is still alive.
    pub fn upgrade(&self) -> Option<Entity<T>> {
        self.inner.upgrade().map(|inner| Entity {
            inner,
            tx: self.tx.clone(),
        })
    }

    /// Update the entity if it is still alive.
    pub fn update<F, R>(&self, f: F) -> Option<crate::Result<R>>
    where
        F: FnOnce(&mut T) -> R,
    {
        self.upgrade().map(|entity| entity.update(f))
    }
}

impl<T: ?Sized + Send + Sync> Clone for Entity<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            tx: self.tx.clone(),
        }
    }
}

impl<T: ?Sized + Send + Sync> Clone for WeakEntity<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            tx: self.tx.clone(),
        }
    }
}

impl<T: Send + Sync> Entity<T> {
    /// Create a new entity with the given initial value.
    pub fn new(value: T) -> Self {
        let (tx, _) = watch::channel(());
        Self {
            inner: Arc::new(Mutex::new(value)),
            tx,
        }
    }
}