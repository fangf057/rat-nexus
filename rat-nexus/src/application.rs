//! High‑level Application abstraction inspired by GPUI.

use crate::component::traits::{Event, Action, Component, AnyComponent};
use crate::state::{Entity, WeakEntity, EntityId};
use ratatui::prelude::*;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::io::{self, stdout};
use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

/// Type-erased storage for application-level shared state.
type StateMap = HashMap<TypeId, Arc<dyn Any + Send + Sync>>;

pub struct AppContext {
    /// The root component to render, if set by the user.
    root: Arc<Mutex<Option<Entity<dyn AnyComponent>>>>,
    /// Internal: Channel to trigger a re-render.
    re_render_tx: mpsc::UnboundedSender<()>,
    /// Internal: Total frames rendered.
    frame_count: Arc<std::sync::atomic::AtomicU64>,
    /// Application-level shared state storage (TypeMap pattern).
    state: Arc<RwLock<StateMap>>,
}

impl Clone for AppContext {
    fn clone(&self) -> Self {
        Self {
            root: Arc::clone(&self.root),
            re_render_tx: mpsc::UnboundedSender::clone(&self.re_render_tx),
            frame_count: Arc::clone(&self.frame_count),
            state: Arc::clone(&self.state),
        }
    }
}

impl AppContext {
    /// Create a new entity with the given value.
    pub fn new_entity<T>(&self, value: T) -> Entity<T>
    where
        T: Send + Sync + 'static,
    {
        Entity::new(value)
    }

    /// Schedule a task to be executed later.
    pub fn spawn<F, Fut>(&self, f: F)
    where
        F: FnOnce(AppContext) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let cx = AppContext::clone(self);
        tokio::spawn(async move {
            f(cx).await;
        });
    }

    /// Spawn a task and return a handle that can be used to cancel it.
    pub fn spawn_task<F, Fut>(&self, f: F) -> crate::task::TaskHandle
    where
        F: FnOnce(AppContext) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let cx = AppContext::clone(self);
        let join_handle = tokio::spawn(async move {
            f(cx).await;
        });
        crate::task::TaskHandle::new(join_handle.abort_handle())
    }

    /// Set the root component of the application.
    fn set_root_component(&self, root: Entity<dyn AnyComponent>) -> crate::Result<()> {
        let mut guard = self.root.lock().map_err(|_| crate::Error::LockPoisoned)?;
        *guard = Some(root);
        self.refresh();
        Ok(())
    }

    /// Set the root component with automatic Arc/RwLock wrapping.
    /// This is a convenience method that handles the boilerplate of wrapping
    /// a component in Arc<RwLock<T>> and creating an Entity.
    ///
    /// # Example
    /// ```ignore
    /// let root = Root::new(cx);
    /// cx.set_root_component(root)?;  // No ugly Arc/RwLock casting needed!
    /// ```
    pub fn set_root<C>(&self, component: C) -> crate::Result<()>
    where
        C: AnyComponent + 'static,
    {
        let locked = Arc::new(RwLock::new(component));
        let root = Entity::from_arc(locked as Arc<RwLock<dyn AnyComponent>>);
        self.set_root_component(root)
    }

    /// Trigger a re-render.
    pub fn refresh(&self) {
        let _ = self.re_render_tx.send(());
    }

    /// Get the total number of frames rendered.
    pub fn frame_count(&self) -> u64 {
        self.frame_count.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Store a value in the application state.
    /// Use this to share state across components.
    ///
    /// # Example
    /// ```ignore
    /// let shared = cx.new_entity(AppState::default());
    /// cx.set(shared);  // Store for later retrieval
    /// ```
    pub fn set<T>(&self, value: T)
    where
        T: Send + Sync + 'static,
    {
        if let Ok(mut guard) = self.state.write() {
            guard.insert(TypeId::of::<T>(), Arc::new(value));
        }
    }

    /// Retrieve a value from the application state.
    /// Returns None if the type was not previously stored.
    ///
    /// # Example
    /// ```ignore
    /// let shared: Entity<AppState> = cx.get().expect("AppState not set");
    /// ```
    pub fn get<T>(&self) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
    {
        self.state
            .read()
            .ok()
            .and_then(|guard| guard.get(&TypeId::of::<T>()).cloned())
            .and_then(|arc| arc.downcast::<T>().ok())
            .map(|arc| (*arc).clone())
    }

    /// Check if a type is stored in the application state.
    pub fn has<T: 'static>(&self) -> bool {
        self.state
            .read()
            .map(|guard| guard.contains_key(&TypeId::of::<T>()))
            .unwrap_or(false)
    }

    /// Get a value from application state, or return a default if not set.
    /// This is safer than `get().expect()` - no panic possible.
    ///
    /// # Example
    /// ```ignore
    /// let shared: Entity<AppState> = cx.get_or_default();
    /// // If AppState was never set, a default instance is created and stored
    /// ```
    pub fn get_or_default<T>(&self) -> Option<T>
    where
        T: Clone + Send + Sync + 'static + Default,
    {
        match self.get::<T>() {
            Some(value) => Some(value),
            None => {
                let default = T::default();
                self.set(default.clone());
                Some(default)
            }
        }
    }

    /// Get a value from application state, or create one using a closure if not set.
    /// More flexible than `get_or_default()` when custom initialization is needed.
    ///
    /// # Example
    /// ```ignore
    /// let shared: Entity<AppState> = cx.get_or_insert_with(|| {
    ///     AppState::new_with_config(config)
    /// });
    /// ```
    pub fn get_or_insert_with<T, F>(&self, f: F) -> Option<T>
    where
        T: Clone + Send + Sync + 'static,
        F: FnOnce() -> T,
    {
        match self.get::<T>() {
            Some(value) => Some(value),
            None => {
                let value = f();
                self.set(value.clone());
                Some(value)
            }
        }
    }
}

/// A specialized context passed to component methods.
/// Inspired by GPUI's Context design - always bound to an entity.
/// Note: For rendering area, use `frame.area()` instead.
pub struct Context<V: ?Sized + Send + Sync> {
    app: AppContext,
    /// The entity this context is bound to. When the context is "cast" to another type
    /// (for calling child components), this becomes None. Use `entity()` for self-reference
    /// and `weak_entity()` for async operations.
    handle: Option<WeakEntity<V>>,
}

// Deref to AppContext for convenient access to app methods
impl<V: ?Sized + Send + Sync> std::ops::Deref for Context<V> {
    type Target = AppContext;

    fn deref(&self) -> &Self::Target {
        &self.app
    }
}

impl<V: ?Sized + Send + Sync> Context<V> {
    /// Create a context bound to an entity. This is the primary constructor.
    pub fn new(app: AppContext, handle: WeakEntity<V>) -> Self {
        Self {
            app,
            handle: Some(handle),
        }
    }

    /// Get a reference to the underlying AppContext.
    /// Use this to access AppContext methods that are shadowed by Context methods
    /// (like spawn/spawn_task for unbound async tasks).
    pub fn app(&self) -> &AppContext {
        &self.app
    }

    /// Subscribe to an entity's changes.
    pub fn subscribe<T>(&mut self, entity: &Entity<T>)
    where T: Send + Sync + 'static
    {
        let mut rx = entity.subscribe();
        let tx = self.app.re_render_tx.clone();
        tokio::spawn(async move {
            while rx.changed().await.is_ok() {
                let _ = tx.send(());
            }
        });
    }

    /// Watch an entity: subscribe to changes and read the current value.
    /// This is a convenience method that combines `subscribe` and `entity.read`.
    pub fn watch<T, F, R>(&mut self, entity: &Entity<T>, f: F) -> Option<R>
    where
        T: Send + Sync + 'static,
        F: FnOnce(&T) -> R,
    {
        self.subscribe(entity);
        entity.read(f).ok()
    }

    /// Spawn an async task with access to the entity's WeakEntity.
    /// This is the GPUI-style spawn that automatically provides a weak reference
    /// to the entity for safe async access.
    ///
    /// # Example
    /// ```ignore
    /// fn save_data(&mut self, cx: &mut Context<Self>) {
    ///     let data = self.data.clone();
    ///     cx.spawn(|weak_self, app| async move {
    ///         tokio::time::sleep(Duration::from_secs(1)).await;
    ///         // Safe: if component was dropped, upgrade() returns None
    ///         if let Some(entity) = weak_self.upgrade() {
    ///             entity.update(|this| this.on_save_complete());
    ///         }
    ///         app.refresh();
    ///     });
    /// }
    /// ```
    ///
    /// # Panics
    /// Panics if the context was not created with a handle (i.e., was cast from another context).
    pub fn spawn<F, Fut>(&self, f: F)
    where
        V: 'static,
        F: FnOnce(WeakEntity<V>, AppContext) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let weak = self.handle.clone()
            .expect("Context::spawn requires a bound entity. Use AppContext::spawn for unbound contexts.");
        let app = AppContext::clone(&self.app);
        tokio::spawn(async move {
            f(weak, app).await;
        });
    }

    /// Spawn a task and return a handle that can be used to cancel it.
    /// Use this with `TaskTracker` for proper lifecycle management.
    ///
    /// # Panics
    /// Panics if the context was not created with a handle.
    pub fn spawn_task<F, Fut>(&self, f: F) -> crate::task::TaskHandle
    where
        V: 'static,
        F: FnOnce(WeakEntity<V>, AppContext) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let weak = self.handle.clone()
            .expect("Context::spawn_task requires a bound entity. Use AppContext::spawn_task for unbound contexts.");
        let app = AppContext::clone(&self.app);
        let join_handle = tokio::spawn(async move {
            f(weak, app).await;
        });
        crate::task::TaskHandle::new(join_handle.abort_handle())
    }

    /// Spawn an unbound async task (no WeakEntity reference).
    /// Use this for background tasks that don't need to access the component.
    /// Delegates to `AppContext::spawn`.
    pub fn spawn_detached<F, Fut>(&self, f: F)
    where
        F: FnOnce(AppContext) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.app.spawn(f)
    }

    /// Spawn an unbound async task with cancellation handle.
    /// Use this for background tasks that don't need to access the component.
    /// Delegates to `AppContext::spawn_task`.
    pub fn spawn_detached_task<F, Fut>(&self, f: F) -> crate::task::TaskHandle
    where
        F: FnOnce(AppContext) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.app.spawn_task(f)
    }

    /// Cast this context to another view type.
    /// Note: The cast context will NOT have a handle. Use `entity.update_with_cx(cx, ...)`
    /// pattern for proper child component lifecycle.
    pub fn cast<U: ?Sized + Send + Sync + 'static>(&self) -> Context<U> {
        Context {
            app: AppContext::clone(&self.app),
            handle: None,
        }
    }

    /// Get the entity ID of the component this context is bound to.
    /// Returns None if the context was cast from another type.
    pub fn entity_id(&self) -> Option<EntityId> {
        self.handle.as_ref().map(|h| h.entity_id())
    }

    /// Get a weak handle to the component this context is bound to.
    /// Returns None if the context was cast from another type.
    /// Use this for async operations to safely check if the entity still exists.
    pub fn weak_entity(&self) -> Option<WeakEntity<V>> {
        self.handle.clone()
    }

    /// Get a strong handle to the component this context is bound to.
    /// Returns None if the context was cast or if the entity was dropped.
    pub fn entity(&self) -> Option<Entity<V>> {
        self.handle.as_ref().and_then(|h| h.upgrade())
    }

    /// Explicitly trigger a re-render.
    pub fn notify(&self) {
        self.app.refresh();
    }
}

/// EventContext for event handling, currently identical to Context but renamed for clarity.
pub type EventContext<V> = Context<V>;

/// Main application handle.
pub struct Application;

impl Application {
    /// Create a new application instance.
    pub fn new() -> Self {
        Self
    }

    /// Run the application with the given closure that receives a context.
    pub fn run<F>(self, setup: F) -> anyhow::Result<()>
    where
        F: FnOnce(&AppContext) -> anyhow::Result<()>,
    {
        let rt = Runtime::new().map_err(|e| anyhow::anyhow!("Failed to start tokio: {}", e))?;
        let (re_render_tx, re_render_rx) = mpsc::unbounded_channel();
        let root = Arc::new(Mutex::new(None));
        let app_context = AppContext {
            root: Arc::clone(&root),
            re_render_tx,
            frame_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            state: Arc::new(RwLock::new(HashMap::new())),
        };

        let _guard = rt.enter();
        setup(&app_context)?;
        drop(_guard);

        let actual_root: Entity<dyn AnyComponent> = {
            let guard = root.lock().map_err(|_| anyhow::anyhow!("Root mutex poisoned"))?;
            guard.as_ref().map(Entity::clone).unwrap_or_else(|| {
                Entity::from_arc(Arc::new(RwLock::new(DummyView)) as Arc<RwLock<dyn AnyComponent>>)
            })
        };

        let result = rt.block_on(async move {
            self.run_loop(app_context, actual_root, re_render_rx).await
        });

        // Ensure we don't hang forever on background tasks (like infinite loops in components)
        rt.shutdown_timeout(Duration::from_millis(100));

        result
    }

    async fn run_loop(&self, app: AppContext, root: Entity<dyn AnyComponent>, re_render_rx: mpsc::UnboundedReceiver<()>) -> anyhow::Result<()> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture, event::EnableFocusChange)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Lifecycle: Call on_mount (first time) and on_enter (entering view) on the root component
        {
            let weak = root.downgrade();
            let mut cx = Context::<dyn AnyComponent>::new(AppContext::clone(&app), weak);
            root.update(|comp| {
                comp.on_mount_any(&mut cx);
                comp.on_enter_any(&mut cx);
            }).map_err(|_| anyhow::anyhow!("Root mutex poisoned during on_mount"))?;
        }

        let result = self.run_app_loop(app, &mut terminal, root, re_render_rx).await;

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture,
            event::DisableFocusChange
        )?;
        terminal.show_cursor()?;

        result
    }

    async fn run_app_loop(
        &self,
        app: AppContext,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
        root: Entity<dyn AnyComponent>,
        mut re_render_rx: mpsc::UnboundedReceiver<()>,
    ) -> anyhow::Result<()> {
        // Initial render
        let _ = app.re_render_tx.send(());

        // Dedicated event polling task to avoid blocking the main loop
        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        tokio::task::spawn_blocking(move || {
            loop {
                // Check if the main loop is still interested in events
                if event_tx.is_closed() {
                    break;
                }

                // Poll at ~60fps (16.67ms) for smooth animations
                match event::poll(Duration::from_millis(16)) {
                    Ok(true) => {
                        if let Ok(e) = event::read() {
                            if event_tx.send(e).is_err() {
                                break;
                            }
                        }
                    }
                    Ok(false) => {}
                    Err(_) => break,
                }
            }
        });

        loop {
            tokio::select! {
                // Prioritize event handling for lower latency
                biased;

                Some(crossterm_event) = event_rx.recv() => {
                    let internal_event = match crossterm_event {
                        CrosstermEvent::Key(key) if key.kind == KeyEventKind::Press => Some(Event::Key(key)),
                        CrosstermEvent::Mouse(mouse) => Some(Event::Mouse(mouse)),
                        CrosstermEvent::Resize(w, h) => Some(Event::Resize(w, h)),
                        CrosstermEvent::FocusGained => Some(Event::FocusGained),
                        CrosstermEvent::FocusLost => Some(Event::FocusLost),
                        CrosstermEvent::Paste(s) => Some(Event::Paste(s)),
                        _ => None,
                    };

                    if let Some(event) = internal_event {
                        let weak = root.downgrade();
                        let mut cx = EventContext::<dyn AnyComponent>::new(AppContext::clone(&app), weak);

                        let action = root.update(|comp| {
                            comp.handle_event_any(event, &mut cx)
                        }).map_err(|_| anyhow::anyhow!("Root mutex poisoned during event"))?;

                        app.refresh(); // Trigger refresh after any event handling

                        if let Some(action) = action {
                            match action {
                                Action::Quit => {
                                    let weak = root.downgrade();
                                    let mut cx = Context::<dyn AnyComponent>::new(AppContext::clone(&app), weak);
                                    root.update(|comp| comp.on_shutdown_any(&mut cx))
                                        .map_err(|_| anyhow::anyhow!("Root mutex poisoned during shutdown"))?;
                                    return Ok(());
                                }
                                _ => {}
                            }
                        }
                    }
                }

                _ = re_render_rx.recv() => {
                    // Drain all pending refresh requests to compact them into a single frame
                    while re_render_rx.try_recv().is_ok() {}

                    let weak = root.downgrade();
                    terminal.draw(|frame| {
                        app.frame_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let mut cx = Context::<dyn AnyComponent>::new(AppContext::clone(&app), weak);
                        root.update(|comp| comp.render_any(frame, &mut cx))
                            .expect("Root mutex poisoned during render");
                    })?;
                }
            }
        }
    }
}

struct DummyView;

impl Component for DummyView {
    fn render(&mut self, frame: &mut ratatui::Frame, _cx: &mut Context<Self>) {
        let paragraph = ratatui::widgets::Paragraph::new("No component set")
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(paragraph, frame.area());
    }
}
