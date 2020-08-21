use crate::backend::EguiBevyBackend;
use bevy::{prelude::*, window::WindowCloseRequested};
use egui::app::RunMode;
use std::{sync::Arc, time::Instant};

struct EguiPluginState {
    start_time: Instant,
    ctx: Arc<egui::Context>,
    raw_input: egui::RawInput,
    runner: EguiBevyBackend,
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
        pixels_per_point: None, // TODO
        ..Default::default()
    }
}

fn egui_system(mut state: ResMut<EguiPluginState>) {
    let egui_start = Instant::now();
    let mut raw_input = state.raw_input.take();
    let time = state.start_time.elapsed().as_nanos() as f64 * 1e-9;
    raw_input.time = time;
    raw_input.seconds_since_midnight = Some(local_time_of_day());

    let mut ui = state.ctx.begin_frame(raw_input);
    // TODO get app
    // app.ui(&mut ui, &mut runner);
    let (output, paint_jobs) = state.ctx.end_frame();

    let frame_time = (Instant::now() - egui_start).as_secs_f64() as f32;
    state.runner.frame_times.add(time, frame_time);

    // TODO
    // painter.paint_jobs(&display, paint_jobs, ctx.texture());

    // at this point egui_glium checks for quit or request_redraw. This is already handle by bevy so we can ignore that

    // handle_output(output, &display, clipboard.as_mut());
}

fn startup(_world: &mut World, resources: &mut Resources) {
    // let storage = FileStorage::from_path(".egui.json".into());
    // let app: egui::DemoApp = egui::app::get_value(&storage, egui::app::APP_KEY).unwrap_or_default();

    let ctx = egui::Context::new();
    let start_time = Instant::now();

    let raw_input = {
        let windows = resources.get::<Windows>().unwrap();
        let window = windows.get_primary().unwrap();

        make_raw_input(window)
    };

    let state = EguiPluginState {
        start_time,
        ctx,
        raw_input,
        runner: EguiBevyBackend::new(RunMode::Continuous), // TODO
    };

    resources.insert(state);
}

pub struct ImguiPlugin;

impl Plugin for ImguiPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<WindowCloseRequestedReader>()
            .add_startup_system(startup.thread_local_system())
            .add_system(egui_system.system())
            .add_system(exit_on_window_close_system.system());
    }
}
