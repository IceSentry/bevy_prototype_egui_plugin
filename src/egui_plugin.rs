use crate::{backend::EguiBevyBackend, painter};
use bevy::{prelude::*, window::WindowCloseRequested};
use egui::app::RunMode;
use std::{sync::Arc, time::Instant};

struct EguiPluginState {
    start_time: Instant,
    frame_start: Instant,
    ctx: Arc<egui::Context>,
    raw_input: Option<egui::RawInput>,
    runner: EguiBevyBackend,
}

pub struct EguiContext {
    pub ui: Option<egui::Ui>,
}

#[derive(Default)]
pub struct WindowCloseRequestedReader {
    event_reader: EventReader<WindowCloseRequested>,
}

pub fn exit_on_window_close_system(
    mut state: Local<WindowCloseRequestedReader>,
    window_close_requested_events: Res<Events<WindowCloseRequested>>,
) {
    if state
        .event_reader
        .iter(&window_close_requested_events)
        .next()
        .is_some()
    {
        // TODO
        // Save app state to a file

        // egui::app::set_value(
        //     &mut storage,
        //     WINDOW_KEY,
        //     &WindowSettings::from_display(&display),
        // );
        // egui::app::set_value(&mut storage, EGUI_MEMORY_KEY, &*ctx.memory());
        // app.on_exit(&mut storage);
        // storage.save();
    }
}

pub fn local_time_of_day() -> f64 {
    use chrono::Timelike;
    let time = chrono::Local::now().time();
    time.num_seconds_from_midnight() as f64 + 1e-9 * (time.nanosecond() as f64)
}

pub fn make_raw_input(window: &Window) -> egui::RawInput {
    egui::RawInput {
        screen_size: { egui::vec2(window.width as f32, window.height as f32) },
        ..Default::default()
    }
}

fn egui_check_windows(mut state: ResMut<EguiPluginState>, windows: Res<Windows>) {
    if state.raw_input.is_none() {
        state.raw_input = Some({
            let window = windows.get_primary().unwrap();
            make_raw_input(window)
        });
    }
}

fn egui_pre_update_system(mut state: ResMut<EguiPluginState>, mut ctx: ResMut<EguiContext>) {
    state.frame_start = Instant::now();
    if let Some(raw_input) = state.raw_input.clone().as_mut() {
        let time = state.start_time.elapsed().as_nanos() as f64 * 1e-9;
        raw_input.time = time;
        raw_input.seconds_since_midnight = Some(local_time_of_day());

        let ui = state.ctx.begin_frame(raw_input.clone());
        ctx.ui = Some(ui);
    }
}

fn egui_post_update_system(mut state: ResMut<EguiPluginState>, ctx: Res<EguiContext>) {
    let frame_time = (Instant::now() - state.frame_start).as_secs_f64() as f32;

    if ctx.ui.is_none() {
        return;
    }

    let (_output, paint_jobs) = state.ctx.end_frame();

    if let Some(raw_input) = state.raw_input.clone() {
        state.runner.frame_times.add(raw_input.time, frame_time);
    }

    painter::paint_jobs(paint_jobs, state.ctx.texture());

    // At this point egui_glium checks for quit() or request_redraw.
    // This is already handled by bevy so we can ignore that

    // TODO
    // handle_output(output, &display, clipboard.as_mut());
}

fn startup(_world: &mut World, resources: &mut Resources) {
    let start_time = Instant::now();

    let ctx = egui::Context::new();

    let state = EguiPluginState {
        start_time,
        frame_start: start_time,
        ctx,
        raw_input: None,
        runner: EguiBevyBackend::new(RunMode::Continuous), // TODO
    };
    let ui = EguiContext { ui: None };

    resources.insert(state);
    resources.insert(ui);
}

pub struct EguiPlugin;

impl Plugin for EguiPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<WindowCloseRequestedReader>()
            .add_startup_system(startup.thread_local_system())
            .add_system_to_stage("pre_update", egui_pre_update_system.system())
            .add_system_to_stage("post_update", egui_post_update_system.system())
            .add_system(egui_check_windows.system())
            .add_system(exit_on_window_close_system.system());
    }
}
