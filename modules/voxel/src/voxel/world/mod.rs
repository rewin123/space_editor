use bevy::{
    math::IVec3,
    prelude::{Component, Plugin},
};
use ndshape::ConstShape3u32;

use super::{storage::ChunkMap, terraingen, Voxel};

/// Systems for dynamically loading / unloading regions (aka chunks) of the world according to camera position.
mod chunks;
pub use chunks::{
    ChunkCommandQueue, ChunkEntities, ChunkLoadRadius, CurrentLocalPlayerChunk, DirtyChunks,
};

mod chunks_anim;
pub mod materials;
mod meshing;
pub mod player;
mod sky;
mod terrain;

/// Registers all resources and systems for simulating and rendering an editable and interactive voxel world.
pub struct VoxelWorldPlugin;

impl Plugin for VoxelWorldPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(ChunkMap::<Voxel, ChunkShape>::new(ChunkShape {}))
            .add_plugins(chunks::VoxelWorldChunkingPlugin)
            .add_plugins(meshing::VoxelWorldMeshingPlugin)
            // ordering of plugin insertion matters here.
            .add_plugins(terraingen::TerrainGeneratorPlugin)
            .add_plugins(terrain::VoxelWorldTerrainGenPlugin)
            .add_plugins(super::material::VoxelMaterialPlugin)
            .add_plugins(super::render::ChunkMaterialPlugin)
            .add_plugins(materials::VoxelWorldBaseMaterialsPlugin)
            .add_plugins(chunks_anim::ChunkAppearanceAnimatorPlugin)
            .add_plugins(bevy_atmosphere::plugin::AtmospherePlugin)
            .add_plugins(player::VoxelWorldPlayerControllerPlugin)
            .add_plugins(sky::InteractiveSkyboxPlugin);
    }
}

pub const CHUNK_LENGTH: u32 = 32;
pub const CHUNK_LENGTH_U: usize = CHUNK_LENGTH as usize;
pub type ChunkShape = ConstShape3u32<CHUNK_LENGTH, CHUNK_LENGTH, CHUNK_LENGTH>;

// A component tagging an entity as a chunk.
#[derive(Component)]
pub struct Chunk(pub IVec3);
