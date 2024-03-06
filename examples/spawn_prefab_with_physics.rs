use bevy::prelude::*;
use kenney_city_editor::prelude::*;
use kcg_editor_ui::ext::bevy_panorbit_camera::{self, PanOrbitCameraPlugin};

//This example loading prefab with bevy_xpbd types

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugins(PrefabPlugin)
        .add_plugins(XpbdPlugin)
        .add_plugins(PanOrbitCameraPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, _assets: Res<AssetServer>) {
    // prefab loaded by adding PrefabLoader component to any entity (it will be parent of prefab) or with prefab bundle
    commands
        .spawn(PrefabBundle::new("scenes/physics_loading_test.scn.ron"))
        .insert(Name::new("Prefab"));

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands
        .spawn(Camera3dBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(bevy_panorbit_camera::PanOrbitCamera::default());
}
