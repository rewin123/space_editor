// Simple event example
// Run command:
// cargo run --example event_example

use bevy::prelude::*;
use space_editor::prelude::*;

#[derive(Event, Default)]
pub struct ToggleSpin;

#[derive(Component, Default, Reflect, Clone)]
#[reflect(Component, Default)]
pub struct Spin(bool);

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugins(SpaceEditorPlugin)
        .add_systems(Startup, simple_editor_setup)
        .add_event::<ToggleSpin>()
        .editor_event::<ToggleSpin>()
        .editor_registry::<Spin>()
        .add_systems(Startup, setup)
        .add_systems(Update, spin_entities)
        .add_systems(Update, handle_spin_event)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((PrefabBundle::new("cube.scn.ron"), Spin(false)));
}

fn spin_entities(mut query: Query<(&mut Transform, &Spin)>, time: Res<Time>) {
    for (mut transform, spin) in query.iter_mut() {
        if spin.0 {
            transform.rotate(Quat::from_rotation_y(time.delta_seconds()));
        }
    }
}

fn handle_spin_event(mut query: Query<&mut Spin>, mut events: EventReader<ToggleSpin>) {
    for mut spin in query.iter_mut() {
        for _ in events.read() {
            spin.0 = !spin.0;
        }
    }
}