use crate::voxel::{
    material::VoxelMaterial,
    materials::{Dirt, Grass, PineLeaves, PineWood, Snow},
    storage::VoxelBuffer,
    terraingen::{common::make_pine_tree, noise},
    ChunkShape, Voxel,
};
use bevy::math::{IVec3, UVec3, Vec2, Vec3Swizzles};
use ilattice::prelude::UVec3 as ILUVec3;

use super::LayeredBiomeTerrainGenerator;

pub struct BasicSnowyPlainsBiomeTerrainGenerator;

impl LayeredBiomeTerrainGenerator for BasicSnowyPlainsBiomeTerrainGenerator {
    fn fill_strata(&self, layer: u32) -> Voxel {
        match layer {
            0 => Snow::into_voxel(),
            1..=2 => Grass::into_voxel(),
            _ => Dirt::into_voxel(),
        }
    }

    fn place_decoration(
        &self,
        key: IVec3,
        pos: UVec3,
        buffer: &mut VoxelBuffer<Voxel, ChunkShape>,
    ) {
        let spawn_chance = noise::rand2to1(
            (pos.xz().as_vec2() + key.xz().as_vec2()) * 0.1,
            Vec2::new(12.989, 78.233),
        );

        if spawn_chance > 0.981 && pos.y <= 13 {
            // this is a stupid hack but a real fix would be to allow terrain decoration to work vertically
            make_pine_tree::<PineWood, PineLeaves>(buffer, ILUVec3::from(pos.to_array()));
        }
    }
}
