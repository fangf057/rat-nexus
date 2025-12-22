//! Application state models demonstrating Entity reactive state management.

/// Global application state shared across all pages.
#[derive(Clone)]
pub struct AppState {
    pub counter: i32,
    pub theme: Theme,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            counter: 0,
            theme: Theme::default(),
        }
    }
}

/// Theme configuration for the application.
#[derive(Clone, Copy, Default, PartialEq)]
pub enum Theme {
    #[default]
    Cyan,
    Green,
    Magenta,
    Yellow,
}

impl Theme {
    pub fn next(&self) -> Self {
        match self {
            Theme::Cyan => Theme::Green,
            Theme::Green => Theme::Magenta,
            Theme::Magenta => Theme::Yellow,
            Theme::Yellow => Theme::Cyan,
        }
    }

    pub fn color(&self) -> ratatui::style::Color {
        match self {
            Theme::Cyan => ratatui::style::Color::Cyan,
            Theme::Green => ratatui::style::Color::Green,
            Theme::Magenta => ratatui::style::Color::Magenta,
            Theme::Yellow => ratatui::style::Color::Yellow,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Theme::Cyan => "Cyan",
            Theme::Green => "Green",
            Theme::Magenta => "Magenta",
            Theme::Yellow => "Yellow",
        }
    }
}

/// State for the System Monitor page.
#[derive(Clone)]
pub struct MonitorState {
    pub cpu_history: Vec<u64>,
    pub memory_history: Vec<u64>,
    pub network_in: Vec<u64>,
    pub network_out: Vec<u64>,
    pub disk_usage: u16,
    pub cpu_cores: Vec<u16>,
    pub processes: Vec<ProcessInfo>,
    pub uptime_secs: u64,
}

#[derive(Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu: f32,
    pub memory: f32,
}

impl Default for MonitorState {
    fn default() -> Self {
        Self {
            cpu_history: vec![0; 60],
            memory_history: vec![0; 60],
            network_in: vec![0; 30],
            network_out: vec![0; 30],
            disk_usage: 45,
            cpu_cores: vec![0; 8],
            processes: vec![
                ProcessInfo { pid: 1, name: "init".into(), cpu: 0.1, memory: 0.5 },
                ProcessInfo { pid: 100, name: "rat-demo".into(), cpu: 2.5, memory: 1.2 },
                ProcessInfo { pid: 200, name: "tokio-rt".into(), cpu: 1.8, memory: 0.8 },
                ProcessInfo { pid: 300, name: "crossterm".into(), cpu: 0.5, memory: 0.3 },
            ],
            uptime_secs: 0,
        }
    }
}
