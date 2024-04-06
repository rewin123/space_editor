use bevy::{prelude::*, window::WindowResolution};
use game_lib::GamePlugin;
use space_editor::SpaceEditorPlugin;
use space_editor_ui::{game_mode_changed, settings::GameModeSettings, simple_editor_setup};

fn main() {
    // Backtrace only supported on unix
    #[cfg(all(feature = "backtrace", target_os = "linux"))]
    unsafe {
        backtrace_on_stack_overflow::enable()
    };

    let mut app = App::new();
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            resizable: true,
            focused: true,
            title: "Space Editor".into(),
            resolution: WindowResolution::new(1600., 900.),
            visible: true,
            ..default()
        }),
        ..default()
    }))
    .add_plugins((SpaceEditorPlugin, GamePlugin))
    .add_systems(Startup, simple_editor_setup)
    .add_systems(
        PreUpdate,
        game_mode_changed.run_if(resource_changed::<GameModeSettings>),
    )
    .run();
}
