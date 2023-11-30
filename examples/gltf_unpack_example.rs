use bevy::prelude::*;
use prefab::SpaceEditorPlugin;
use shared::EditorEvent;
use space_editor::{editor::EditorPlugin, simple_editor_setup};

fn main() {
    App::default()
        .add_plugins((DefaultPlugins, EditorPlugin))
        .add_plugins(SpaceEditorPlugin::default())
        .add_systems(Startup, simple_editor_setup)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut editor_events: EventWriter<EditorEvent>) {
    editor_events.send(EditorEvent::LoadGltfAsPrefab(
        "low_poly_fighter_2.gltf".to_string(),
    ));
}
