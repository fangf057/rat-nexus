#[derive(Default, Clone)]
pub struct AppState {
    pub counter: i32,
    pub history: Vec<u64>,
}

#[derive(Clone)]
pub struct LocalState {
    pub layout_horizontal: bool,
    pub logs: Vec<String>,
    pub progress: u16,
    pub is_working: bool,
    pub current_time: String,
    pub system_load: Vec<u64>,
    pub pulse_inc: u8, // Decay value for visual feedback
    pub pulse_dec: u8, // Decay value for visual feedback
    pub fps: f64,
}

impl Default for LocalState {
    fn default() -> Self {
        Self {
            layout_horizontal: false,
            logs: vec!["Initialized".to_string()],
            progress: 0,
            is_working: false,
            current_time: "--:--:--".to_string(),
            system_load: vec![0; 20],
            pulse_inc: 0,
            pulse_dec: 0,
            fps: 0.0,
        }
    }
}
