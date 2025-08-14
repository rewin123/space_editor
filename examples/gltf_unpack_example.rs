use bevy::prelude::*;
use space_editor::prelude::*;

fn main() {
    App::default()
        .add_plugins((DefaultPlugins, SpaceEditorPlugin))
        .add_systems(Startup, simple_editor_setup)
        .add_systems(Startup, setup)
        .register_type::<PrefabMarker>()


        .run();
}

fn setup(
    mut editor_events: EventWriter<EditorEvent>,
    mut commands: Commands,
) {
    
    let test_parent = commands.spawn((
        Name::from("Test Parent"),
        PrefabMarker,
        Transform::default(),
        Visibility::default()
    )).id();
    
    editor_events.write(EditorEvent::LoadGltfAsPrefab{
        path: "models/colone.glb".to_string(),
        parent: Some(test_parent)
    });
}
