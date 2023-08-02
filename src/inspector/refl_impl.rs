
use bevy::{prelude::*, ecs::world::unsafe_world_cell::UnsafeWorldCell};
use bevy_egui::*;

pub fn reflect_name(
    ui :  &mut egui::Ui,
    name : &mut Name,
    hash : &str,
    label : &str,
    setup_updated : &mut dyn FnMut(),
    world : &mut UnsafeWorldCell
) {
    ui.horizontal(|ui| {
        name.mutate(|s| {
            ui.label(label);
            ui.add(egui::TextEdit::singleline(s));
        });
    });
}

pub fn reflect_string(
    ui :  &mut egui::Ui,
    name : &mut String,
    hash : &str,
    label : &str,
    setup_updated : &mut dyn FnMut(),
    world : &mut UnsafeWorldCell
) {
    ui.horizontal(|ui| {
        ui.label(label);
        if ui.add(egui::TextEdit::singleline(name)).changed() {
            setup_updated();
        }
    });
}