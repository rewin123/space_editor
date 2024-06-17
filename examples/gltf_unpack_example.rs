use bevy::prelude::*;
use space_editor::prelude::*;

fn main() {
    App::default()
        .add_plugins((DefaultPlugins, SpaceEditorPlugin))
        .add_systems(Startup, simple_editor_setup)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut editor_events: EventWriter<EditorEvent>) {
    editor_events.send(EditorEvent::LoadGltfAsPrefab(
        "models/low_poly_fighter_2.gltf".to_string(),
    ));
}
