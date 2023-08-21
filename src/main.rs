use std::f32::consts::PI;

use bevy_mod_picking::prelude::RaycastPickCamera;
use space_editor::prelude::*;
use bevy::prelude::*;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugins(SpaceEditorPlugin::default())
        .add_systems(Startup, setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    
   // light
    commands.spawn(DirectionalLightBundle {
        directional_light : DirectionalLight { shadows_enabled: true, ..default() },
        transform: Transform::from_xyz(4.0, 8.0, 4.0).with_rotation(Quat::from_rotation_x(-PI / 4.)),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    }).insert(PanOrbitCamera::default())
    .insert(RaycastPickCamera::default());
}