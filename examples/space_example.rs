use space_editor::prelude::*;
use bevy::prelude::*;


fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugins(SpaceEditorPlugin::default())
        .add_systems(Startup, simple_editor_setup)
        .run();
}
