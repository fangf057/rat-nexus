//! Example TUI application using the rat-setup framework.

mod model;
mod pages;
mod app;

use rat_nexus::Application;
use crate::model::AppState;
use crate::app::Root;

fn main() -> anyhow::Result<()> {
    let app = Application::new();

    app.run(move |cx| {
        let shared_state = cx.new_entity(AppState::default());
        let root = Root::new(shared_state, cx);
        let root = std::sync::Arc::new(std::sync::Mutex::new(root));
        cx.set_root(root)?;

        // Optional: Startup task
        cx.spawn(|_| async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        });

        Ok(())
    })
}
