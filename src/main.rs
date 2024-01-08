use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    #[cfg(feature = "editor")]
    {
        use space_editor::SpaceEditorPlugin;
        use space_editor_ui::simple_editor_setup;
        app.add_plugins(SpaceEditorPlugin)
            .add_systems(Startup, simple_editor_setup);
    }
    #[cfg(not(feature = "editor"))]
    {
        use space_prefab::plugins::PrefabPlugin;
        app.add_plugins(PrefabPlugin);
    }
    #[cfg(feature = "experimental")]
    {
        app.add_plugins(terrain_gen::TerraingenInspectorPlugin);
    }
    app.run();
}
