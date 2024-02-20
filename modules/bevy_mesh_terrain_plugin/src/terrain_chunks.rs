use bevy::prelude::*;
use bevy_mesh_terrain::{
    chunk::{Chunk, ChunkData, TerrainChunkMesh},
    edit::TerrainCommandEvent,
    terrain::TerrainData,
    terrain_config::TerrainConfig,
};
use space_editor_ui::prelude::*;

pub struct TerrainChunksPlugin;

impl Plugin for TerrainChunksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, add_brushable_terrain_components);
    }
}

#[derive(Component)]
pub struct BrushableTerrain {}

fn add_brushable_terrain_components(
    mut commands: Commands,
    chunks_query: Query<(Entity, &TerrainChunkMesh), Without<BrushableTerrain>>,
) {
    for (entity, _) in chunks_query.iter() {
        commands.entity(entity).insert(BrushableTerrain {});
    }
}
