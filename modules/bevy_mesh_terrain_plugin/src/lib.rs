pub mod terrain_brush;
pub mod terrain_chunks;
pub mod terrain_tab;
pub mod terrain_tool;

use bevy::prelude::*;
use bevy_mesh_terrain::terrain::TerrainViewer;

use space_editor_ui::{
    ext::bevy_mod_picking::backends::raycast::bevy_mod_raycast::DefaultRaycastingPlugin, prelude::*,
};

pub struct BevyMeshTerrainPlugin;

impl Plugin for BevyMeshTerrainPlugin {
    fn build(&self, app: &mut App) {
        app
            // .init_resource::<TerrainTools>()
            .add_plugins(DefaultRaycastingPlugin)
            .add_plugins((
                bevy_mesh_terrain::TerrainMeshPlugin::default(),
                terrain_tab::TerrainTabPlugin,
                terrain_tool::TerrainToolPlugin,
                terrain_chunks::TerrainChunksPlugin,
                terrain_brush::TerrainBrushPlugin,
            ));

        app.add_systems(Update, add_viewer_to_editor_cams);
    }
}

fn add_viewer_to_editor_cams(
    mut commands: Commands,
    query: Query<Entity, With<EditorCameraMarker>>,
) {
    for e in query.iter() {
        commands.entity(e).insert(TerrainViewer::default());
    }
}
