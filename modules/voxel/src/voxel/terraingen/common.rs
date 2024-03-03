use bevy::math::{IVec3, Vec3};
use ilattice::{glam::UVec2, glam::UVec3, prelude::Extent};

use crate::voxel::{
    material::VoxelMaterial,
    materials::{Bedrock, Rock, Water},
    sdf,
    storage::VoxelBuffer,
    ChunkShape, Voxel, CHUNK_LENGTH, CHUNK_LENGTH_U,
};

use super::noise::Heightmap;

/// Generate the world bottom border for a chunk.
pub fn terrain_generate_world_bottom_border(buffer: &mut VoxelBuffer<Voxel, ChunkShape>) {
    buffer.fill_extent(
        Extent::from_min_and_shape(UVec3::ZERO, UVec3::new(CHUNK_LENGTH, 2, CHUNK_LENGTH)),
        Bedrock::into_voxel(),
    )
}

/// Carve the general terrain shape for a chunk.
pub fn terrain_carve_heightmap(
    buffer: &mut VoxelBuffer<Voxel, ChunkShape>,
    key: IVec3,
    heighmap: &Heightmap<CHUNK_LENGTH_U, CHUNK_LENGTH_U>,
) {
    // drown the terrain under sea level.
    if key.y <= 96 {
        buffer.fill_extent(
            Extent::from_min_and_shape(UVec3::ZERO, UVec3::splat(CHUNK_LENGTH)),
            Water::into_voxel(),
        );
    }

    // carve the terrain.
    Extent::from_min_and_shape(UVec2::ZERO, UVec2::new(CHUNK_LENGTH, CHUNK_LENGTH))
        .iter2()
        .for_each(|pos| {
            let local_height = heighmap
                .get(pos.into())
                .checked_sub(key.y as u32)
                .unwrap_or_default()
                .min(CHUNK_LENGTH);

            for h in 0..local_height {
                *buffer.voxel_at_mut([pos.x, h, pos.y].into()) = Rock::into_voxel();
            }
        });
}

pub fn make_pine_tree<T: VoxelMaterial, L: VoxelMaterial>(
    buffer: &mut VoxelBuffer<Voxel, ChunkShape>,
    origin: UVec3,
) {
    let origin = Vec3::from(origin.as_vec3().to_array());
    Extent::from_min_and_shape(UVec3::ZERO, UVec3::splat(CHUNK_LENGTH)) //may want to calculate an extent encompassing the tree instead of iterating over the complete 32^3 volume
        .iter3()
        .map(|x| Vec3::from_array(x.as_vec3().to_array()))
        .map(|position| {
            let trunk_distance =
                sdf::sdf_capped_cylinder(position - (origin + 2.0 * Vec3::Y), 1.5, 8.0) < 0.;
            let leaves_distance =
                sdf::sdf_vcone(position - (origin + 6.0 * Vec3::Y), 7.0, 17.0) < 0.;
            (trunk_distance, leaves_distance, position)
        })
        .map(|(trunk_distance, leaves_distance, position)| {
            (
                trunk_distance,
                leaves_distance,
                UVec3::from(position.as_uvec3().to_array()),
            )
        })
        .for_each(|(trunk_distance, leaves_distance, position)| {
            if trunk_distance {
                *buffer.voxel_at_mut(position) = T::into_voxel()
            }

            if leaves_distance {
                *buffer.voxel_at_mut(position) = L::into_voxel()
            }
        })
}

/// Make a tree using SDF functions
pub fn make_tree<T: VoxelMaterial, L: VoxelMaterial>(
    buffer: &mut VoxelBuffer<Voxel, ChunkShape>,
    origin: UVec3,
) {
    let origin = Vec3::from(origin.as_vec3().to_array());
    Extent::from_min_and_shape(UVec3::ZERO, UVec3::splat(CHUNK_LENGTH)) //may want to calculate an extent encompassing the tree instead of iterating over the complete 32^3 volume
        .iter3()
        .map(|x| Vec3::from_array(x.as_vec3().to_array()))
        .map(|position| {
            let trunk_distance =
                sdf::sdf_capped_cylinder(position - (origin + 2.0 * Vec3::Y), 1.5, 8.0) < 0.;
            let leaves_distance = sdf::sdf_sphere(position - (origin + 14.0 * Vec3::Y), 6.0) < 0.;
            (trunk_distance, leaves_distance, position)
        })
        .map(|(trunk_distance, leaves_distance, position)| {
            (
                trunk_distance,
                leaves_distance,
                UVec3::from(position.as_uvec3().to_array()),
            )
        })
        .for_each(|(trunk_distance, leaves_distance, position)| {
            if trunk_distance {
                *buffer.voxel_at_mut(position) = T::into_voxel()
            }

            if leaves_distance {
                *buffer.voxel_at_mut(position) = L::into_voxel()
            }
        });
}

pub fn make_rock<V: VoxelMaterial>(
    buffer: &mut VoxelBuffer<Voxel, ChunkShape>,
    origin: UVec3,
    size: f32,
) {
    let origin = Vec3::from(origin.as_vec3().to_array());
    Extent::from_min_and_shape(UVec3::ZERO, UVec3::splat(CHUNK_LENGTH)) //may want to calculate an extent encompassing the tree instead of iterating over the complete 32^3 volume
        .iter3()
        .map(|x| Vec3::from_array(x.as_vec3().to_array()))
        .map(|position| {
            let trunk_distance = sdf::sdf_sphere(position - origin, size) < 0.;
            (trunk_distance, position)
        })
        .map(|(rock_distance, position)| {
            (rock_distance, UVec3::from(position.as_uvec3().to_array()))
        })
        .for_each(|(rock, position)| {
            if rock {
                *buffer.voxel_at_mut(position) = V::into_voxel()
            }
        });
}
