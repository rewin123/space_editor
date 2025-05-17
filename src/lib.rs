//created just for loading together crates

/// Contains all the functions/structs/plugins of space_editor
pub mod prelude {
    pub use crate::SpaceEditorPlugin;
    pub use space_editor_ui::prelude::*;
}

pub use space_editor_ui;
pub use space_prefab;

use bevy::prelude::*;
use space_editor_ui::prelude::*;

/// This is the main plugin, connecting it will allow you to use all the functions of space_editor
pub struct SpaceEditorPlugin;

impl bevy::app::Plugin for SpaceEditorPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(space_editor_ui::EditorPlugin);
        app.add_plugins(AllEditorTabsPlugin);
    }
}



pub struct AllEditorTabsPlugin;


impl Plugin for AllEditorTabsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(space_editor_game_view::GameViewPlugin);
        app.editor_tab_by_trait(space_editor_game_view::GameViewTab::default());
    }
}


