use rat_nexus::{Component, Context, EventContext, Event, Action, Route, Entity, AppContext};
use crate::model::AppState;
use crate::pages::{Menu, MonitorPage, TimerPage, ParticlesPage, FlappyPage, TicTacToePage};

pub struct Root {
    current: Route,
    history: Vec<Route>,
    menu: Menu,
    monitor: MonitorPage,
    timer: TimerPage,
    particles: ParticlesPage,
    flappy: FlappyPage,
    tictactoe: TicTacToePage,
}

impl Root {
    pub fn new(shared_state: Entity<AppState>, cx: &AppContext) -> Self {
        Self {
            current: "menu".to_string(),
            history: Vec::new(),
            menu: Menu::new(shared_state.clone()),
            monitor: MonitorPage::new(shared_state, cx),
            timer: TimerPage::new(cx),
            particles: ParticlesPage::new(cx),
            flappy: FlappyPage::new(cx),
            tictactoe: TicTacToePage::new(cx),
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
    fn on_mount(&mut self, cx: &mut Context<Self>) {
        self.menu.on_mount(&mut cx.cast());
        self.monitor.on_mount(&mut cx.cast());
        self.timer.on_mount(&mut cx.cast());
        self.particles.on_mount(&mut cx.cast());
        self.flappy.on_mount(&mut cx.cast());
        self.tictactoe.on_mount(&mut cx.cast());
    }

    fn on_enter(&mut self, cx: &mut Context<Self>) {
        match self.current.as_str() {
            "monitor" => self.monitor.on_enter(&mut cx.cast()),
            "timer" => self.timer.on_enter(&mut cx.cast()),
            "particles" => self.particles.on_enter(&mut cx.cast()),
            "flappy" => self.flappy.on_enter(&mut cx.cast()),
            "tictactoe" => self.tictactoe.on_enter(&mut cx.cast()),
            _ => self.menu.on_enter(&mut cx.cast()),
        }
    }

    fn render(&mut self, frame: &mut ratatui::Frame, cx: &mut Context<Self>) {
        match self.current.as_str() {
            "monitor" => self.monitor.render(frame, &mut cx.cast()),
            "timer" => self.timer.render(frame, &mut cx.cast()),
            "particles" => self.particles.render(frame, &mut cx.cast()),
            "flappy" => self.flappy.render(frame, &mut cx.cast()),
            "tictactoe" => self.tictactoe.render(frame, &mut cx.cast()),
            _ => self.menu.render(frame, &mut cx.cast()),
        }
    }

    fn handle_event(&mut self, event: Event, cx: &mut EventContext<Self>) -> Option<Action> {
        let current = self.current.clone();
        let action = match current.as_str() {
            "monitor" => self.monitor.handle_event(event, &mut cx.cast()),
            "timer" => self.timer.handle_event(event, &mut cx.cast()),
            "particles" => self.particles.handle_event(event, &mut cx.cast()),
            "flappy" => self.flappy.handle_event(event, &mut cx.cast()),
            "tictactoe" => self.tictactoe.handle_event(event, &mut cx.cast()),
            _ => self.menu.handle_event(event, &mut cx.cast()),
        };

        if let Some(action) = action {
            match action {
                Action::Navigate(route) => {
                    // Lifecycle: on_exit current → navigate → on_enter new
                    match current.as_str() {
                        "monitor" => self.monitor.on_exit(&mut cx.cast()),
                        "timer" => self.timer.on_exit(&mut cx.cast()),
                        "particles" => self.particles.on_exit(&mut cx.cast()),
                        "flappy" => self.flappy.on_exit(&mut cx.cast()),
                        "tictactoe" => self.tictactoe.on_exit(&mut cx.cast()),
                        _ => self.menu.on_exit(&mut cx.cast()),
                    }
                    self.navigate(route);
                    match self.current.as_str() {
                        "monitor" => self.monitor.on_enter(&mut cx.cast()),
                        "timer" => self.timer.on_enter(&mut cx.cast()),
                        "particles" => self.particles.on_enter(&mut cx.cast()),
                        "flappy" => self.flappy.on_enter(&mut cx.cast()),
                        "tictactoe" => self.tictactoe.on_enter(&mut cx.cast()),
                        _ => self.menu.on_enter(&mut cx.cast()),
                    }
                    None
                }
                Action::Back => {
                    match current.as_str() {
                        "monitor" => self.monitor.on_exit(&mut cx.cast()),
                        "timer" => self.timer.on_exit(&mut cx.cast()),
                        "particles" => self.particles.on_exit(&mut cx.cast()),
                        "flappy" => self.flappy.on_exit(&mut cx.cast()),
                        "tictactoe" => self.tictactoe.on_exit(&mut cx.cast()),
                        _ => self.menu.on_exit(&mut cx.cast()),
                    }
                    if self.go_back() {
                        match self.current.as_str() {
                            "monitor" => self.monitor.on_enter(&mut cx.cast()),
                            "timer" => self.timer.on_enter(&mut cx.cast()),
                            "particles" => self.particles.on_enter(&mut cx.cast()),
                            "flappy" => self.flappy.on_enter(&mut cx.cast()),
                            "tictactoe" => self.tictactoe.on_enter(&mut cx.cast()),
                            _ => self.menu.on_enter(&mut cx.cast()),
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
