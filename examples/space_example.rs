use bevy::{
    log::{Level, LogPlugin},
    prelude::*,
};
use ext::bevy_inspector_egui::quick::WorldInspectorPlugin;
use space_editor::prelude::*;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            level: Level::TRACE,
            ..default()
        }))
        .add_plugins(SpaceEditorPlugin)
        .add_systems(Startup, simple_editor_setup)
        .run();
}
