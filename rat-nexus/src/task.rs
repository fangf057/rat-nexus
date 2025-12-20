//! Task management utilities for component lifecycle.
//!
//! Provides `TaskHandle` for cancellable async tasks and `TaskTracker` for
//! managing multiple tasks that should be cancelled together (e.g., on component exit).

use tokio::task::AbortHandle;

/// A handle to a spawned task that can be aborted.
#[derive(Debug)]
pub struct TaskHandle {
    abort_handle: AbortHandle,
}

impl TaskHandle {
    /// Create a new TaskHandle from an AbortHandle.
    pub fn new(abort_handle: AbortHandle) -> Self {
        Self { abort_handle }
    }

    /// Abort the task. The task will be cancelled at the next await point.
    pub fn abort(&self) {
        self.abort_handle.abort();
    }

    /// Check if the task has finished (either completed or aborted).
    pub fn is_finished(&self) -> bool {
        self.abort_handle.is_finished()
    }
}

/// A collection of task handles that can be cancelled together.
///
/// Useful for components that spawn multiple background tasks that should
/// all be cancelled when the component exits.
///
/// # Example
/// ```ignore
/// struct MyComponent {
///     tasks: TaskTracker,
/// }
///
/// impl Component for MyComponent {
///     fn on_mount(&mut self, cx: &mut Context<Self>) {
///         // Spawn a task and track it
///         let handle = cx.spawn_task(|_| async move {
///             loop {
///                 // do something
///                 tokio::time::sleep(Duration::from_secs(1)).await;
///             }
///         });
///         self.tasks.track(handle);
///     }
///
///     fn on_exit(&mut self, _cx: &mut Context<Self>) {
///         // Cancel all tracked tasks
///         self.tasks.abort_all();
///     }
/// }
/// ```
#[derive(Debug, Default)]
pub struct TaskTracker {
    handles: Vec<TaskHandle>,
}

impl TaskTracker {
    /// Create a new empty TaskTracker.
    pub fn new() -> Self {
        Self { handles: Vec::new() }
    }

    /// Track a task handle. The task will be aborted when `abort_all` is called.
    pub fn track(&mut self, handle: TaskHandle) {
        // Clean up finished tasks while adding new ones
        self.handles.retain(|h| !h.is_finished());
        self.handles.push(handle);
    }

    /// Abort all tracked tasks.
    pub fn abort_all(&mut self) {
        for handle in self.handles.drain(..) {
            handle.abort();
        }
    }

    /// Get the number of active (non-finished) tracked tasks.
    pub fn active_count(&self) -> usize {
        self.handles.iter().filter(|h| !h.is_finished()).count()
    }

    /// Check if there are any active tasks.
    pub fn has_active_tasks(&self) -> bool {
        self.handles.iter().any(|h| !h.is_finished())
    }
}

impl Drop for TaskTracker {
    fn drop(&mut self) {
        // Automatically abort all tasks when the tracker is dropped
        self.abort_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_task_handle_abort() {
        let handle = tokio::spawn(async {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });
        let task_handle = TaskHandle::new(handle.abort_handle());
        assert!(!task_handle.is_finished());
        task_handle.abort();
        // Give it a moment to register the abort
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        assert!(task_handle.is_finished());
    }

    #[tokio::test]
    async fn test_task_tracker() {
        let mut tracker = TaskTracker::new();

        let h1 = tokio::spawn(async { loop { tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; } });
        let h2 = tokio::spawn(async { loop { tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; } });

        tracker.track(TaskHandle::new(h1.abort_handle()));
        tracker.track(TaskHandle::new(h2.abort_handle()));

        assert_eq!(tracker.active_count(), 2);

        tracker.abort_all();
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        assert_eq!(tracker.active_count(), 0);
    }
}
