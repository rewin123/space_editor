use bevy::prelude::*;
use editor::simple_editor_setup;
use space_editor::SpaceEditorPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, SpaceEditorPlugin))
        .add_systems(Startup, simple_editor_setup)
        .run();
}
