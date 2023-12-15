use bevy::prelude::*;

use terraingen::{systems::*, TerraingenPlugin};

use self::inspector::TerrainGenView;

use super::{editor_tab::EditorTabName, EditorUiAppExt};

pub mod inspector;

pub struct TerraingenInspectorPlugin;

impl Plugin for TerraingenInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TerraingenPlugin)
            .editor_tab_by_trait(
                EditorTabName::Other("Terrain Generator".to_string()),
                TerrainGenView,
            )
            .add_systems(OnEnter(shared::EditorState::Editor), draw_terrain)
            .add_systems(Update, redraw_terrain);
    }
}
