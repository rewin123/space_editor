use bevy::{pbr::CascadeShadowConfigBuilder, prelude::*};
use bevy_mod_picking::{backends::raycast::RaycastPickable, PickableBundle};
use shared::EditorCameraMarker;

/// Module contains all editor UI logic and components
pub mod editor;
/// Optional editor extensions (like activate `bevy_xpbd` support in editor)
pub mod optional;

/// This method prepare default lights and camera for editor UI. You can create own conditions for your editor and use this method how example
pub fn simple_editor_setup(mut commands: Commands) {
    commands.insert_resource(bevy::pbr::DirectionalLightShadowMap { size: 4096 });
    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        cascade_shadow_config: CascadeShadowConfigBuilder::default().into(),
        ..default()
    });

    // camera
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(bevy_panorbit_camera::PanOrbitCamera::default())
        .insert(EditorCameraMarker)
        .insert(PickableBundle::default())
        .insert(RaycastPickable);

    bevy_debug_grid::spawn_floor_grid(commands);
}
