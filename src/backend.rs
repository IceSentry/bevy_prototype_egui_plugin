use egui::app::{Backend, RunMode};

pub struct EguiBevyBackend {
    pub frame_times: egui::MovementTracker<f32>,
    quit: bool,
    run_mode: RunMode,
}

impl EguiBevyBackend {
    pub fn new(run_mode: RunMode) -> Self {
        Self {
            frame_times: egui::MovementTracker::new(1000, 1.0),
            quit: false,
            run_mode,
        }
    }
}

impl Backend for EguiBevyBackend {
    fn run_mode(&self) -> RunMode {
        self.run_mode
    }

    fn set_run_mode(&mut self, run_mode: RunMode) {
        self.run_mode = run_mode;
    }

    fn cpu_time(&self) -> f32 {
        self.frame_times.average().unwrap_or_default()
    }

    fn fps(&self) -> f32 {
        1.0 / self.frame_times.mean_time_interval().unwrap_or_default()
    }

    fn quit(&mut self) {
        self.quit = true;
    }
}
