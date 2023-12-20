use bevy::prelude::*;
use space_shared::EditorState;

use crate::{
    mesh::systems::{draw_terrain, redraw_terrain},
    TerraingenPlugin,
};

use self::ui::TerrainGenView;

use space_editor_ui::{editor_tab::EditorTabName, EditorUiAppExt};

pub mod ui;

pub struct TerraingenInspectorPlugin;

impl Plugin for TerraingenInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TerraingenPlugin)
            .editor_tab_by_trait(
                EditorTabName::Other("Terrain Generator".to_string()),
                TerrainGenView,
            )
            .add_systems(OnEnter(EditorState::Editor), draw_terrain)
            .add_systems(Update, redraw_terrain);
    }
}
