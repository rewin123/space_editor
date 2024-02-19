pub mod terrain_tab;
pub mod terrain_tool;

use bevy::prelude::*;
use space_editor_ui::prelude::*;
use bevy_mesh_terrain::terrain::TerrainViewer;

pub struct BevyMeshTerrainPlugin;

impl Plugin for BevyMeshTerrainPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            bevy_mesh_terrain::TerrainMeshPlugin::default(),
            terrain_tab::TerrainTabPlugin,
            terrain_tool::TerrainToolPlugin
        ));

        app.add_systems(Update, add_viewer_to_editor_cams);
    }
}

fn add_viewer_to_editor_cams(
    mut commands: Commands,
    mut query: Query<Entity, With<EditorCameraMarker>>,
) {
    for e in query.iter() {
        commands.entity(e).insert(TerrainViewer::default());
    }
}