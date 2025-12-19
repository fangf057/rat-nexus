//! High‑level Application abstraction inspired by GPUI.

use crate::component::traits::{Event, Action, Component, AnyComponent};
use crate::state::{Entity, WeakEntity};
use ratatui::prelude::*;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event as CrosstermEvent, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, stdout};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

/// Application context providing access to global services.
#[derive(Clone)]
pub struct AppContext {
    /// The root component to render, if set by the user.
    root: Arc<Mutex<Option<Arc<Mutex<dyn AnyComponent>>>>>,
    /// Internal: Channel to trigger a re-render.
    re_render_tx: mpsc::UnboundedSender<()>,
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
        let cx = self.clone();
        tokio::spawn(async move {
            f(cx).await;
        });
    }

    /// Set the root component of the application.
    pub fn set_root(&self, root: Arc<Mutex<dyn AnyComponent>>) -> crate::Result<()> {
        let mut guard = self.root.lock().map_err(|_| crate::Error::LockPoisoned)?;
        *guard = Some(root);
        self.refresh();
        Ok(())
    }

    /// Trigger a re-render.
    pub fn refresh(&self) {
        let _ = self.re_render_tx.send(());
    }
}

/// A specialized context passed to component methods.
pub struct Context<V: ?Sized + Send + Sync> {
    pub app: AppContext,
    pub area: Rect,
    pub handle: Option<WeakEntity<V>>,
}

impl<V: ?Sized + Send + Sync> Context<V> {
    pub fn new(app: AppContext, area: Rect) -> Self {
        Self {
            app,
            area,
            handle: None,
        }
    }

    pub fn with_handle(app: AppContext, area: Rect, handle: WeakEntity<V>) -> Self {
        Self {
            app,
            area,
            handle: Some(handle),
        }
    }

    /// Access the underlying AppContext.
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
            while let Ok(_) = rx.changed().await {
                let _ = tx.send(());
            }
        });
    }

    /// Spawn a task with access to the entity's weak handle.
    pub fn spawn<F, Fut>(&self, f: F)
    where
        V: 'static,
        F: FnOnce(WeakEntity<V>, AppContext) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        if let Some(handle) = self.handle.clone() {
            let app = self.app.clone();
            tokio::spawn(async move {
                f(handle, app).await;
            });
        }
    }

    /// Cast this context to another view type.
    pub fn cast<U: ?Sized + Send + Sync + 'static>(&self) -> Context<U> {
        Context {
            app: self.app.clone(),
            area: self.area,
            handle: unsafe { std::mem::transmute_copy(&self.handle) },
        }
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
            root: root.clone(),
            re_render_tx,
        };

        let _guard = rt.enter();
        setup(&app_context)?;
        drop(_guard);

        let actual_root = {
            let guard = root.lock().map_err(|_| anyhow::anyhow!("Root mutex poisoned"))?;
            guard.clone().unwrap_or_else(|| Arc::new(Mutex::new(DummyView)))
        };

        rt.block_on(async move {
            self.run_loop(app_context, actual_root, re_render_rx).await
        })
    }

    async fn run_loop(&self, app: AppContext, root: Arc<Mutex<dyn AnyComponent>>, re_render_rx: mpsc::UnboundedReceiver<()>) -> anyhow::Result<()> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture, event::EnableFocusChange)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Lifecycle: Call on_init on the root component
        {
            let size = terminal.size()?;
            let area = Rect::new(0, 0, size.width, size.height);
            let mut guard = root.lock().map_err(|_| anyhow::anyhow!("Root mutex poisoned during on_init"))?;
            let mut cx = Context::<dyn AnyComponent>::new(app.clone(), area);
            guard.on_init_any(&mut cx);
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
        root: Arc<Mutex<dyn AnyComponent>>,
        mut re_render_rx: mpsc::UnboundedReceiver<()>,
    ) -> anyhow::Result<()> {
        // Initial render
        let _ = app.re_render_tx.send(());

        loop {
            tokio::select! {
                _ = re_render_rx.recv() => {
                    terminal.draw(|frame| {
                        let area = frame.area();
                        let mut cx = Context::<dyn AnyComponent>::new(app.clone(), area);
                        // In a real GPUI-like app, the root would be an Entity<dyn AnyComponent>.
                        // For now, we just pass the context.
                        let mut guard = root.lock().expect("Root mutex poisoned during render");
                        guard.render_any(frame, &mut cx);
                    })?;
                }
                event_ready = async { event::poll(Duration::from_millis(100)) } => {
                    if let Ok(true) = event_ready {
                        let crossterm_event = event::read()?;
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
                            let size = terminal.size()?;
                            let area = Rect::new(0, 0, size.width, size.height);
                            let mut cx = EventContext::<dyn AnyComponent>::new(app.clone(), area);
                            
                            let mut guard = root.lock().map_err(|_| anyhow::anyhow!("Root mutex poisoned during event"))?;
                            let action = guard.handle_event_any(event, &mut cx);
                            app.refresh(); // Trigger refresh after any event handling

                            if let Some(action) = action {
                                match action {
                                    Action::Quit => {
                                        // Lifecycle: Call on_shutdown
                                        guard.on_shutdown_any(&mut cx);
                                        return Ok(());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

struct DummyView;

impl Component for DummyView {
    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        let paragraph = ratatui::widgets::Paragraph::new("No component set")
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(paragraph, cx.area);
    }
}