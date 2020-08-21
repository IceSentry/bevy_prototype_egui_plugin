mod backend;
mod egui_plugin;
mod painter;

use bevy::prelude::*;

fn main() {
    App::build().add_default_plugins().run();
}
