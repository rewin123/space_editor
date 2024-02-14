use bevy::{prelude::*, window::WindowResolution};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resizable: true,
            focused: true,
            fit_canvas_to_parent: true,
            title: "Space Editor".into(),
            resolution: WindowResolution::new(1600., 900.),
            visible: true,
            ..default()
        }),
        ..default()
    }));
    #[cfg(feature = "editor")]
    {
        use space_editor::SpaceEditorPlugin;
        use space_editor_ui::{game_mode_changed, settings::GameModeSettings, simple_editor_setup};

        app.add_plugins(SpaceEditorPlugin)
            .add_systems(Startup, simple_editor_setup)
            .add_systems(
                PreUpdate,
                game_mode_changed.run_if(resource_changed::<GameModeSettings>()),
            );
    }
    app.run();
}
