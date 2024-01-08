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
    #[cfg(feature = "experimental")] {
        use terrain_gen::TerraingenInspectorPlugin;
        app.add_plugins(TerraingenInspectorPlugin);
    }
    app.run();
}
