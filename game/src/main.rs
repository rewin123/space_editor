use bevy::{
    prelude::*,
    window::{WindowMode, WindowResolution},
};
use game_lib::GamePlugin;
use space_prefab::prelude::{PrefabBundle, PrefabPlugin};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resizable: true,
            focused: true,
            title: "Your Game".into(),
            resolution: WindowResolution::new(1600., 900.),
            visible: true,
            mode: WindowMode::Fullscreen,
            ..default()
        }),
        ..default()
    }))
    .add_plugins((PrefabPlugin, GamePlugin))
    .add_systems(Startup, setup)
    .run();
}

fn setup(mut commands: Commands, _assets: Res<AssetServer>) {
    // prefab loaded by adding PrefabLoader component to any entity (it will be parent of prefab) or with prefab bundle
    commands
        .spawn(PrefabBundle::new("scenes/cube.scn.ron"))
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
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
