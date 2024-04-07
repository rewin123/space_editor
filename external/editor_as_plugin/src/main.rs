use bevy::{prelude::*, window::WindowResolution};
use space_editor_ui::{
    prelude::{MeshPrimitive3dPrefab, PrefabMarker},
    simple_editor_setup,
};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resizable: true,
            focused: true,
            title: "Test Editor".into(),
            resolution: WindowResolution::new(1600., 900.),
            visible: true,
            ..default()
        }),
        ..default()
    }))
    .add_plugins(space_editor_ui::EditorPlugin)
    .add_systems(Startup, simple_editor_setup)
    .add_systems(Startup, |mut commands: Commands| {
        commands.spawn((
            PrefabMarker,
            MeshPrimitive3dPrefab::Cube(1.2),
            Name::new("Cube".to_string()),
            Transform::default(),
            VisibilityBundle::default(),
        ));
    })
    .run();
}
