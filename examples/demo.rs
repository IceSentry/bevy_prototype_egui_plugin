use bevy::prelude::*;
use bevy_prototype_egui_plugin::egui_plugin::{EguiContext, EguiPlugin};
use egui::label;

fn egui_demo_system(mut ctx: ResMut<EguiContext>) {
    if let Some(ui) = ctx.ui.as_mut() {
        // ui.collapsing("About Egui bevy prototype", |ui| {
        //     ui.add(label!(
        //         "This is an experimental plugin to add support for the immediate mode GUI library Egui." 
        //     ));
        //     ui.horizontal(|ui| {
        //         ui.label("Egui home page:");
        //         ui.hyperlink("https://github.com/emilk/egui");
        //     });
        //     ui.horizontal(|ui| {
        //         ui.label("bevy home page:");
        //         ui.hyperlink("https://github.com/bevyengine/bevy");
        //     });
        //     ui.horizontal(|ui| {
        //         ui.label("Egui bevy plugin home page:");
        //         ui.hyperlink("https://github.com/IceSentry/bevy_prototype_egui_plugin");
        //     });
        // });

        Window::new("Debug").show(ui.ctx(), |ui| {
            ui.label(format!("Hello, world {}", 123));
            if ui.button("Save").clicked {
                my_save_function();
            }
            ui.text_edit(&mut my_string);
            ui.add(Slider::f32(&mut value, 0.0..=1.0).text("float"));
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
