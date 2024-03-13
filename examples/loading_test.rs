use bevy::prelude::*;
use space_editor::prelude::*;

fn main() {
    App::default()
        .add_plugins(DefaultPlugins)
        .add_plugins(SpaceEditorPlugin)
        .add_systems(Startup, simple_editor_setup)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, _assets: Res<AssetServer>) {
    // prefab loaded by adding PrefabLoader component to any entity (it will be parent of prefab) or with prefab bundle
    commands
        .spawn(PrefabBundle::new("scenes/load_test.scn.ron"))
        .insert(Name::new("Prefab"));
}
