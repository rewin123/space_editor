use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    #[cfg(feature = "editor")]
    {
        use editor::simple_editor_setup;
        use space_editor::SpaceEditorPlugin;
        app.add_plugins(SpaceEditorPlugin)
            .add_systems(Startup, simple_editor_setup);
    }
    app.run();
}
