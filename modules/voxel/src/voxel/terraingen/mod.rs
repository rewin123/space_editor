use float_ord::FloatOrd;
use std::{collections::BTreeMap, sync::RwLock};

use bevy::{
    math::{IVec3, Vec3Swizzles},
    prelude::Plugin,
};
use once_cell::sync::Lazy;

use self::{
    biomes::{BiomeTerrainGenerator, IntoBoxedTerrainGenerator},
    common::terrain_generate_world_bottom_border,
    noise::{generate_heightmap_data, Heightmap},
};

use super::{storage::VoxelBuffer, ChunkShape, Voxel, CHUNK_LENGTH_U};

mod biomes;

/// noise functions ported over from C / GLSL code
pub mod noise;

/// common functions used by all terrain generators
pub mod common;

// Terrain generator singleton.
pub static TERRAIN_GENERATOR: Lazy<RwLock<TerrainGenerator>> = Lazy::new(Default::default);

#[derive(Default)]
pub struct TerrainGenerator {
    biomes_map: BTreeMap<FloatOrd<f32>, Box<dyn BiomeTerrainGenerator>>,
}

impl TerrainGenerator {
    pub fn register_biome_generator(
        &mut self,
        chance: f32,
        biome: Box<dyn BiomeTerrainGenerator>,
    ) -> &mut Self {
        self.biomes_map.insert(FloatOrd(chance), biome);
        self
    }

    //returns the biome with the closest temp / humidity
    #[allow(clippy::borrowed_box)]
    fn biome_at(&self, chunk_key: IVec3) -> &Box<dyn BiomeTerrainGenerator> {
        const BIOME_INVSCALE: f32 = 0.001;

        let coords = noise::voronoi(chunk_key.xzy().truncate().as_vec2() * BIOME_INVSCALE);
        let p = FloatOrd(noise::rand2to1i(coords));

        self.biomes_map
            .range(..=p)
            .last()
            .map_or(self.biomes_map.first_key_value().unwrap().1, |x| x.1)
    }

    pub fn generate(&self, chunk_key: IVec3, buffer: &mut VoxelBuffer<Voxel, ChunkShape>) {
        let biome = self.biome_at(chunk_key);
        let noise = generate_heightmap_data(chunk_key, CHUNK_LENGTH_U);

        let noise_map = Heightmap::<CHUNK_LENGTH_U, CHUNK_LENGTH_U>::from_slice(&noise);

        common::terrain_carve_heightmap(buffer, chunk_key, &noise_map);

        biome.carve_terrain(chunk_key, noise_map, buffer);
        biome.decorate_terrain(chunk_key, noise_map, buffer);

        if chunk_key.y == 0 {
            terrain_generate_world_bottom_border(buffer);
        }
    }
}

pub struct TerrainGeneratorPlugin;

impl Plugin for TerrainGeneratorPlugin {
    fn build(&self, _: &mut bevy::prelude::App) {
        TERRAIN_GENERATOR
            .write()
            .unwrap()
            .register_biome_generator(
                0.0f32,
                biomes::BasicPlainsBiomeTerrainGenerator.into_boxed_generator(),
            )
            .register_biome_generator(
                0.8f32,
                biomes::BasicDesertBiomeTerrainGenerator.into_boxed_generator(),
            )
            .register_biome_generator(
                3.21,
                biomes::BasicSnowyPlainsBiomeTerrainGenerator.into_boxed_generator(),
            );
    }
}
