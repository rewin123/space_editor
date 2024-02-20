use bevy::prelude::*;
use bevy_mesh_terrain::{
    edit::TerrainCommandEvent, terrain::TerrainData, terrain_config::TerrainConfig,
};
use space_editor_ui::prelude::*;

pub struct TerrainTabPlugin;

impl Plugin for TerrainTabPlugin {
    fn build(&self, app: &mut App) {
        app.editor_tab_by_trait(
            EditorTabName::Other("Terrain".to_string()),
            TerrainTab::default(),
        );
    }
}

#[derive(Resource, Default)]
pub struct TerrainTab {}

impl EditorTab for TerrainTab {
    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::egui::Ui,
        commands: &mut Commands,
        world: &mut World,
    ) {
        if ui.button("Spawn new terrain").clicked() {
            commands.spawn((
                SpatialBundle::default(),
                TerrainConfig::load_from_file("assets/default_terrain/terrain_config.ron").unwrap(),
                TerrainData::new(),
                PrefabMarker,
            ));
        }

        if ui.button("Save All Chunks (Ctrl+S)").clicked() {
            world.send_event(TerrainCommandEvent::SaveAllChunks(true, true, true))
        }

        if ui.button("Save Splat and Height").clicked() {
            world.send_event(TerrainCommandEvent::SaveAllChunks(true, true, false))
        }
    }

    fn title(&self) -> bevy_inspector_egui::egui::WidgetText {
        "Terrain".into()
    }
}
