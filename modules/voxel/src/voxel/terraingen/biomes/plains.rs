use crate::voxel::{
    material::VoxelMaterial,
    materials::{Dirt, Grass, Leaves, Rock, Wood},
    storage::VoxelBuffer,
    terraingen::{
        common::{make_rock, make_tree},
        noise,
    },
    ChunkShape, Voxel,
};
use bevy::math::{IVec3, UVec3, Vec2, Vec3Swizzles};
use ilattice::prelude::UVec3 as ILUVec3;

use super::LayeredBiomeTerrainGenerator;

pub struct BasicPlainsBiomeTerrainGenerator;

impl LayeredBiomeTerrainGenerator for BasicPlainsBiomeTerrainGenerator {
    fn fill_strata(&self, layer: u32) -> Voxel {
        match layer {
            0..=1 => Grass::into_voxel(),
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

        let grass_blade_height = ((noise::rand2to1(
            (pos.xz().as_vec2() + key.xz().as_vec2()) * 0.1,
            Vec2::new(42.478_2, 8_472.243),
        ) * 100.) as u32)
            .rem_euclid(4);

        if grass_blade_height > 1 && pos.y <= 29 {
            for y in 0..grass_blade_height {
                let position = ILUVec3::from_array(pos.to_array()) + ILUVec3::new(0, y, 0);
                *buffer.voxel_at_mut(position) = Grass::into_voxel();
            }
        }

        // Let's put some rock boulders in the plains to populate a lil bit
        let rock_spawn_chance = noise::rand2to1(
            (pos.xz().as_vec2() + key.xz().as_vec2()) * 0.1,
            Vec2::new(72_845.48, 8_472.243),
        );

        if rock_spawn_chance > 0.995 {
            let rock_size = (1.0f32 - rock_spawn_chance) * 1000.0;
            make_rock::<Rock>(buffer, ILUVec3::from(pos.to_array()), rock_size);
        }

        if spawn_chance > 0.981 && pos.y <= 13 {
            // this is a stupid hack but a real fix would be to allow terrain decoration to work vertically
            make_tree::<Wood, Leaves>(buffer, ILUVec3::from(pos.to_array()));
        }
    }
}
