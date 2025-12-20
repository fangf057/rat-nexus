use rat_nexus::{Component, Context, EventContext, Event, Action, Route, Entity, AppContext};
use crate::model::{AppState, LocalState};
use crate::pages::{Menu, CounterPage, SnakePage};

// A simple root component that switches between menu and pages
pub struct Root {
    current: Route,
    history: Vec<Route>,
    menu: Menu,
    page_a: CounterPage,
    page_b: CounterPage,
    snake: SnakePage,
}

impl Root {
    pub fn new(shared_state: Entity<AppState>, cx: &AppContext) -> Self {
        Self {
            current: "menu".to_string(),
            history: Vec::new(),
            menu: Menu::new(shared_state.clone()),
            page_a: CounterPage::new("Page A", shared_state.clone(), cx.new_entity(LocalState::default())),
            page_b: CounterPage::new("Page B", shared_state, cx.new_entity(LocalState::default())),
            snake: SnakePage::new(cx),
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
    fn on_init(&mut self, cx: &mut Context<Self>) {
        {
            let mut cx = cx.cast::<Menu>();
            self.menu.on_init(&mut cx);
        }
        {
            let mut cx = cx.cast::<CounterPage>();
            self.page_a.on_init(&mut cx);
        }
        {
            let mut cx = cx.cast::<CounterPage>();
            self.page_b.on_init(&mut cx);
        }
        {
            let mut cx = cx.cast::<SnakePage>();
            self.snake.on_init(&mut cx);
        }
    }

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
            "snake" => {
                let mut cx = cx.cast::<SnakePage>();
                self.snake.render(frame, &mut cx);
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
            "snake" => {
                let mut cx = cx.cast::<SnakePage>();
                self.snake.handle_event(event, &mut cx)
            }
            _ => {
                let mut cx = cx.cast::<Menu>();
                self.menu.handle_event(event, &mut cx)
            }
        };

        if let Some(action) = action {
            match action {
                Action::Navigate(route) => {
                    // Lifecycle: Call on_exit for current page
                    match current.as_str() {
                        "page_a" => self.page_a.on_exit(&mut cx.cast()),
                        "page_b" => self.page_b.on_exit(&mut cx.cast()),
                        "snake" => self.snake.on_exit(&mut cx.cast()),
                        _ => self.menu.on_exit(&mut cx.cast()),
                    }
                    self.navigate(route);
                    // Lifecycle: Call on_init for new page (optional, but consistent)
                    match self.current.as_str() {
                        "page_a" => self.page_a.on_init(&mut cx.cast()),
                        "page_b" => self.page_b.on_init(&mut cx.cast()),
                        "snake" => self.snake.on_init(&mut cx.cast()),
                        _ => self.menu.on_init(&mut cx.cast()),
                    }
                    None
                }
                Action::Back => {
                    // Lifecycle: Call on_exit
                    match current.as_str() {
                        "page_a" => self.page_a.on_exit(&mut cx.cast()),
                        "page_b" => self.page_b.on_exit(&mut cx.cast()),
                        "snake" => self.snake.on_exit(&mut cx.cast()),
                        _ => self.menu.on_exit(&mut cx.cast()),
                    }
                    if self.go_back() {
                        // Lifecycle: Call on_init
                        match self.current.as_str() {
                            "page_a" => self.page_a.on_init(&mut cx.cast()),
                            "page_b" => self.page_b.on_init(&mut cx.cast()),
                            "snake" => self.snake.on_init(&mut cx.cast()),
                            _ => self.menu.on_init(&mut cx.cast()),
                        }
                    }
                    None
                }
                Action::Quit => Some(Action::Quit),
                Action::Noop => None,
            }
        } else {
            None
        }
    }
}
