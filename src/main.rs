//! Example TUI application using the ratatui scaffolding with gpui‑style Application.

mod component;
mod router;
mod state;
mod application;

use component::Component;
use component::traits::{Event, Action};
use router::traits::Route;
use state::Entity;
use application::{Application, Context, EventContext};
use crossterm::event::KeyCode;

// Define application state (Model)
#[derive(Default, Clone)]
struct AppState {
    counter: i32,
}

// Define a menu component
struct Menu {
    selected: usize,
    options: Vec<(&'static str, Route)>,
}

impl Menu {
    fn new() -> Self {
        Self {
            selected: 0,
            options: vec![
                ("Counter Page A", "page_a".to_string()),
                ("Counter Page B", "page_b".to_string()),
                ("Exit", "exit".to_string()),
            ],
        }
    }
}

impl Component for Menu {
    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        use ratatui::widgets::{Block, Borders, List, ListItem};
        use ratatui::style::{Style, Modifier};

        let block = Block::default().title("Main Menu").borders(Borders::ALL);
        let inner_area = block.inner(cx.area);
        frame.render_widget(block, cx.area);

        let items: Vec<ListItem> = self.options.iter()
            .enumerate()
            .map(|(i, (label, _))| {
                let style = if i == self.selected {
                    Style::default().add_modifier(Modifier::REVERSED)
                } else {
                    Style::default()
                };
                ListItem::new(*label).style(style)
            })
            .collect();

        let list = List::new(items)
            .highlight_symbol("> ")
            .highlight_style(Style::default().add_modifier(Modifier::BOLD));
        frame.render_widget(list, inner_area);
    }

    fn handle_event(&mut self, event: Event, _cx: &mut EventContext<Self>) -> Option<Action> {
        match event {
            Event::Key(key) if key.code == KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                None
            }
            Event::Key(key) if key.code == KeyCode::Down => {
                if self.selected < self.options.len() - 1 {
                    self.selected += 1;
                }
                None
            }
            Event::Key(key) if key.code == KeyCode::Enter => {
                let (_, route) = &self.options[self.selected];
                if route == "exit" {
                    Some(Action::Quit)
                } else {
                    Some(Action::Navigate(route.clone()))
                }
            }
            Event::Key(key) if key.code == KeyCode::Char('q') => {
                Some(Action::Quit)
            }
            _ => None,
        }
    }
}

// Define a counter page component using Entity
struct CounterPage {
    title: &'static str,
    state: Entity<AppState>,
}

impl CounterPage {
    fn new(title: &'static str, state: Entity<AppState>) -> Self {
        Self { title, state }
    }
}

impl Component for CounterPage {
    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        let counter = self.state.get().counter;
        let text = format!("{} - Counter: {}", self.title, counter);
        let paragraph = ratatui::widgets::Paragraph::new(text)
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(paragraph, cx.area);
    }

    fn handle_event(&mut self, event: Event, _cx: &mut EventContext<Self>) -> Option<Action> {
        match event {
            Event::Key(key) if key.code == KeyCode::Char('j') => {
                let mut state = self.state.get_mut();
                state.counter += 1;
                None
            }
            Event::Key(key) if key.code == KeyCode::Char('k') => {
                let mut state = self.state.get_mut();
                state.counter -= 1;
                None
            }
            Event::Key(key) if key.code == KeyCode::Char('m') => {
                Some(Action::Navigate("menu".to_string()))
            }
            Event::Key(key) if key.code == KeyCode::Esc => {
                Some(Action::Back)
            }
            Event::Key(key) if key.code == KeyCode::Char('q') => {
                Some(Action::Quit)
            }
            _ => None,
        }
    }
}

// A simple root component that switches between menu and pages
struct Root {
    current: Route,
    history: Vec<Route>,
    menu: Menu,
    page_a: CounterPage,
    page_b: CounterPage,
}

impl Root {
    fn new(shared_state: Entity<AppState>) -> Self {
        Self {
            current: "menu".to_string(),
            history: Vec::new(),
            menu: Menu::new(),
            page_a: CounterPage::new("Page A", shared_state.clone()),
            page_b: CounterPage::new("Page B", shared_state),
        }
    }

    fn navigate(&mut self, route: Route) {
        if self.current != route {
            self.history.push(self.current.clone());
            self.current = route;
        }
    }

    fn go_back(&mut self) -> bool {
        if let Some(prev) = self.history.pop() {
            self.current = prev;
            true
        } else {
            false
        }
    }
}

impl Component for Root {
    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        let current = self.current.clone();
        match current.as_str() {
            "page_a" => {
                let mut cx = cx.cast::<CounterPage>();
                self.page_a.render(frame, &mut cx);
            }
            "page_b" => {
                let mut cx = cx.cast::<CounterPage>();
                self.page_b.render(frame, &mut cx);
            }
            _ => {
                let mut cx = cx.cast::<Menu>();
                self.menu.render(frame, &mut cx);
            }
        }
    }

    fn handle_event(&mut self, event: Event, cx: &mut EventContext<Self>) -> Option<Action> {
        let current = self.current.clone();
        let action = match current.as_str() {
            "page_a" => {
                let mut cx = cx.cast::<CounterPage>();
                self.page_a.handle_event(event, &mut cx)
            }
            "page_b" => {
                let mut cx = cx.cast::<CounterPage>();
                self.page_b.handle_event(event, &mut cx)
            }
            _ => {
                let mut cx = cx.cast::<Menu>();
                self.menu.handle_event(event, &mut cx)
            }
        };

        if let Some(action) = action {
            match action {
                Action::Navigate(route) => {
                    self.navigate(route);
                    None
                }
                Action::Back => {
                    if self.go_back() {
                        None
                    } else {
                        None
                    }
                }
                Action::Quit => Some(Action::Quit),
                Action::Noop => None,
            }
        } else {
            None
        }
    }
}

fn main() -> std::io::Result<()> {
    let app = Application::new();

    app.run(move |cx| {
        let shared_state = cx.new_entity(AppState::default());
        let root = Root::new(shared_state);
        let root = std::sync::Arc::new(std::sync::Mutex::new(root));
        cx.set_root(root);

        cx.spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        });

        Ok(())
    })
}
