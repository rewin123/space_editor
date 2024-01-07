// Simple event example
// Open the ["Event Dispacther" tab](https://github.com/rewin123/space_editor/pull/163) to send the "ToggleSpin" event.
// Run command:
// cargo run --example custom_event

use bevy::prelude::*;
use space_editor::prelude::*;

#[derive(Event, Default, Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct ToggleSpin {
    speed: f32,
}

#[derive(Component, Default, Reflect, Clone)]
#[reflect(Component, Default)]
pub struct Spin(bool, f32);

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugins(SpaceEditorPlugin)
        .add_systems(Startup, simple_editor_setup)
        .add_event::<ToggleSpin>()
        .editor_registry_event::<ToggleSpin>()
        .editor_registry::<Spin>()
        .add_systems(Startup, setup)
        .add_systems(Update, spin_entities)
        .add_systems(Update, handle_spin_event)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        PrefabBundle::new("cube.scn.ron"),
        Spin(false, 0.),
        PrefabMarker,
    ));
}

fn spin_entities(mut query: Query<(&mut Transform, &Spin)>, time: Res<Time>) {
    for (mut transform, spin) in query.iter_mut() {
        if spin.0 {
            transform.rotate(Quat::from_rotation_y(spin.1 * time.delta_seconds()));
        }
    }
}

fn handle_spin_event(mut query: Query<&mut Spin>, mut events: EventReader<ToggleSpin>) {
    for event in events.read() {
        for mut spin in query.iter_mut() {
            spin.0 = !spin.0;
            spin.1 = event.speed;
        }
    }
}
