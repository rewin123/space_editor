use std::ops::Div;

use bevy::math::{IVec3, UVec3};
use ilattice::{glam::UVec2, prelude::Extent};

use crate::voxel::{
    material::VoxelMaterial,
    materials::{Dirt, Grass},
    storage::VoxelBuffer,
    terraingen::noise::Heightmap,
    ChunkShape, Voxel, CHUNK_LENGTH, CHUNK_LENGTH_U,
};

use super::BiomeTerrainGenerator;

/// A biome terrain generator that applies a set of layers on top of the terrain.
pub trait LayeredBiomeTerrainGenerator: BiomeTerrainGenerator {
    /// The height function to use for applying the biome material layers on top of the terrain.
    fn fill_strata(&self, layer: u32) -> Voxel {
        match layer {
            0..=1 => Grass::into_voxel(),
            _ => Dirt::into_voxel(),
        }
    }

    /// Numbers of material layers to apply on top of the terrain
    fn num_layers(&self) -> u32 {
        8
    }

    fn place_decoration(
        &self,
        _key: IVec3,
        _pos: UVec3,
        _buffer: &mut VoxelBuffer<Voxel, ChunkShape>,
    ) {
    }
}

impl<T: LayeredBiomeTerrainGenerator> BiomeTerrainGenerator for T {
    fn carve_terrain(
        &self,
        chunk_key: IVec3,
        heightmap: Heightmap<CHUNK_LENGTH_U, CHUNK_LENGTH_U>,
        buffer: &mut VoxelBuffer<Voxel, ChunkShape>,
    ) {
        Extent::from_min_and_shape(UVec2::ZERO, UVec2::splat(CHUNK_LENGTH))
            .iter2()
            .for_each(|pos| {
                let height = heightmap.get(pos.into());
                // we only want to apply surface layer decoration on top of the surface chunk
                if height.div(CHUNK_LENGTH) == (chunk_key.y as u32).div(CHUNK_LENGTH) {
                    let local_height = height.rem_euclid(CHUNK_LENGTH);

                    for h in 0..=self.num_layers() {
                        let remaining_height = local_height.checked_sub(h);

                        if let Some(uh) = remaining_height {
                            *buffer.voxel_at_mut([pos.x, uh, pos.y].into()) = self.fill_strata(h)
                        }
                    }
                }
            });
    }

    fn decorate_terrain(
        &self,
        chunk_key: IVec3,
        heightmap: Heightmap<CHUNK_LENGTH_U, CHUNK_LENGTH_U>,
        buffer: &mut VoxelBuffer<Voxel, ChunkShape>,
    ) {
        if chunk_key.y <= 96 {
            return;
        }

        Extent::from_min_and_shape(UVec2::ZERO, UVec2::splat(CHUNK_LENGTH))
            .iter2()
            .for_each(|pos| {
                let height = heightmap.get(pos.into());

                if height.div(CHUNK_LENGTH) == (chunk_key.y as u32).div(CHUNK_LENGTH) {
                    let local_height = height.rem_euclid(CHUNK_LENGTH);
                    self.place_decoration(chunk_key, [pos.x, local_height, pos.y].into(), buffer);
                }
            });
    }
}
