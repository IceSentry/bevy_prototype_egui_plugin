mod backend;
mod egui_plugin;
mod painter;

use bevy::prelude::*;
use egui_plugin::EguiPlugin;

fn main() {
    App::build()
        .add_default_plugins()
        .add_plugin(EguiPlugin)
        .run();
}
