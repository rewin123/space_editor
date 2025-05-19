use bevy::prelude::*;
use space_editor_ui::prelude::*;
use space_editor_tabs::prelude::*;
use space_editor_game_view::*;


fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(EditorPlugin)
        .add_plugins(GameViewPlugin)
        .run();
}

