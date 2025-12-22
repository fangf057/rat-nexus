//! Example TUI application using the rat-setup framework.

mod model;
mod pages;
mod app;

use rat_nexus::Application;
use crate::app::Root;

fn main() -> anyhow::Result<()> {
    let app = Application::new();

    app.run(move |cx| {
        cx.set_root(Root::new())?;
        Ok(())
    })
}
