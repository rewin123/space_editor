use bevy::prelude::*;
use prefab::SpaceEditorPlugin;
use space_editor::{editor::EditorPlugin, simple_editor_setup};

fn main() {
    App::default()
        .add_plugins((DefaultPlugins, EditorPlugin))
        .add_plugins(SpaceEditorPlugin::default())
        .add_systems(Startup, simple_editor_setup)
        .run();
}
