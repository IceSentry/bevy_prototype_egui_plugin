mod backend;
mod egui_plugin;
mod painter;

use bevy::prelude::*;
use egui::label;
use egui_plugin::{EguiContext, EguiPlugin};

fn egui_demo_system(mut ctx: ResMut<EguiContext>) {
    if let Some(ui) = ctx.ui.as_mut() {
        ui.collapsing("About Egui", |ui| {
            ui.add(label!(
                "Egui is an experimental immediate mode GUI written in Rust."
            ));

            ui.horizontal(|ui| {
                ui.label("Project home page:");
                ui.hyperlink("https://github.com/emilk/egui");
            });
        });
    }
}

fn main() {
    App::build()
        .add_default_plugins()
        .add_plugin(EguiPlugin)
        .add_system(egui_demo_system.system())
        .run();
}
