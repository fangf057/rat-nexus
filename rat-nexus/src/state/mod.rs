use std::num::NonZeroU64;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock, Weak};
use tokio::sync::watch;

/// Global counter for generating unique entity IDs.
static NEXT_ENTITY_ID: AtomicU64 = AtomicU64::new(1);

/// A unique identifier for an entity across the application.
/// Guaranteed to be unique across the entire application lifetime.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(NonZeroU64);

impl EntityId {
    /// Generate a new unique EntityId.
    ///
    /// # Panics
    /// Panics if more than 2^64-1 entities are created (theoretically impossible in practice).
    /// Consider implementing overflow recovery in production systems handling extreme scale.
    fn next() -> Self {
        let id = NEXT_ENTITY_ID.fetch_add(1, Ordering::Relaxed);
        // SAFETY: We start at 1 and only increment, so it's never zero.
        Self(NonZeroU64::new(id).unwrap_or_else(|| {
            panic!("EntityId overflow: created more than 2^64-1 entities")
        }))
    }

    /// Get the raw u64 value.
    pub fn as_u64(&self) -> u64 {
        self.0.get()
    }
}

impl std::fmt::Debug for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EntityId({})", self.0)
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Shared state wrapper with RwLock for efficient concurrent access.
/// - Use read() for read-heavy workloads (no contention)
/// - Use write() for mutations (exclusive access)
/// - Allows multiple concurrent readers or one exclusive writer
pub type SharedState<T> = Arc<RwLock<T>>;

/// Entity handle, inspired by GPUI.
/// Each entity has a unique ID and can be subscribed to for change notifications.
pub struct Entity<T: ?Sized + Send + Sync> {
    id: EntityId,
    pub(crate) inner: SharedState<T>,
    tx: watch::Sender<()>,
}

/// A weak handle to an entity.
pub struct WeakEntity<T: ?Sized + Send + Sync> {
    id: EntityId,
    pub(crate) inner: Weak<RwLock<T>>,
    tx: watch::Sender<()>,
}

impl<T: ?Sized + Send + Sync> Entity<T> {
    /// Get the unique ID of this entity.
    pub fn entity_id(&self) -> EntityId {
        self.id
    }

    /// Update the inner value using a closure and notify subscribers.
    pub fn update<F, R>(&self, f: F) -> crate::Result<R>
    where
        F: FnOnce(&mut T) -> R,
    {
        let mut guard = self.inner.write().map_err(|_| crate::Error::LockPoisoned)?;
        let res = f(&mut *guard);
        drop(guard);
        let _ = self.tx.send(());
        Ok(res)
    }

    /// Update the inner value with a Context bound to this entity.
    /// This is the GPUI-style update that provides a properly bound Context for async operations.
    ///
    /// # Example
    /// ```ignore
    /// // Instead of:
    /// let mut cx = cx.cast::<MyComponent>();
    /// entity.update(|c| c.handle_event(event, &mut cx));
    ///
    /// // Use:
    /// entity.update_with_cx(&cx.app, |c, cx| c.handle_event(event, cx));
    /// ```
    pub fn update_with_cx<F, R>(&self, app: &crate::AppContext, f: F) -> crate::Result<R>
    where
        T: 'static,
        F: FnOnce(&mut T, &mut crate::Context<T>) -> R,
    {
        let weak = self.downgrade();
        let mut cx = crate::Context::new(app.clone(), weak);
        let mut guard = self.inner.write().map_err(|_| crate::Error::LockPoisoned)?;
        let res = f(&mut *guard, &mut cx);
        drop(guard);
        let _ = self.tx.send(());
        Ok(res)
    }

    /// Read the inner value using a closure (non-blocking for concurrent readers).
    pub fn read<F, R>(&self, f: F) -> crate::Result<R>
    where
        F: FnOnce(&T) -> R,
    {
        let guard = self.inner.read().map_err(|_| crate::Error::LockPoisoned)?;
        Ok(f(&*guard))
    }

    /// Downgrade this entity to a weak handle.
    pub fn downgrade(&self) -> WeakEntity<T> {
        WeakEntity {
            id: self.id,
            inner: Arc::downgrade(&self.inner),
            tx: watch::Sender::clone(&self.tx),
        }
    }

    /// Subscribe to changes of this entity.
    pub fn subscribe(&self) -> watch::Receiver<()> {
        self.tx.subscribe()
    }
}

impl<T: ?Sized + Send + Sync> WeakEntity<T> {
    /// Get the unique ID of this entity.
    pub fn entity_id(&self) -> EntityId {
        self.id
    }

    /// Upgrade this weak handle to a strong handle, if the entity is still alive.
    pub fn upgrade(&self) -> Option<Entity<T>> {
        self.inner.upgrade().map(|inner| Entity {
            id: self.id,
            inner,
            tx: watch::Sender::clone(&self.tx),
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
            id: self.id,
            inner: Arc::clone(&self.inner),
            tx: watch::Sender::clone(&self.tx),
        }
    }
}

impl<T: ?Sized + Send + Sync> Clone for WeakEntity<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            inner: Weak::clone(&self.inner),
            tx: watch::Sender::clone(&self.tx),
        }
    }
}

impl<T: Send + Sync> Entity<T> {
    /// Create a new entity with the given initial value.
    pub fn new(value: T) -> Self {
        let (tx, _) = watch::channel(());
        Self {
            id: EntityId::next(),
            inner: Arc::new(RwLock::new(value)),
            tx,
        }
    }
}

impl<T: ?Sized + Send + Sync> Entity<T> {
    /// Create an entity from an existing Arc<RwLock<T>>.
    /// This is useful for creating Entity<dyn Trait> from coerced Arc types.
    pub fn from_arc(inner: Arc<RwLock<T>>) -> Self {
        let (tx, _) = watch::channel(());
        Self {
            id: EntityId::next(),
            inner,
            tx,
        }
    }
}
