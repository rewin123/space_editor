use bevy::prelude::*;
use space_editor::prelude::*;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugins(SpaceEditorPlugin)
        .add_systems(Startup, simple_editor_setup)
        .run();
}
