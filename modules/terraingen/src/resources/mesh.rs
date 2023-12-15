use bevy::{prelude::*, render::mesh::Indices};

use crate::biomes::Biomes;

#[derive(Reflect, Default, Debug, Clone)]
pub struct NoiseValues {
    pub height: f64,
    pub moisture: f64,
    pub temperature: f64,
}

#[derive(Reflect, Default, Debug, Clone, Resource)]
#[reflect(Resource, Default)]
pub struct TerrainMesh {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub biomes: Vec<Biomes>,
    pub normals: Vec<Vec3>,
}

impl TerrainMesh {
    pub fn new(
        vertices: Vec<Vec3>,
        indices: Vec<u32>,
        biomes: Vec<Biomes>,
        normals: Vec<Vec3>,
    ) -> Self {
        Self {
            vertices,
            indices,
            biomes,
            normals,
        }
    }

    pub fn indices(&self) -> Indices {
        // TODO: Optimize meshes/mesh count
        // https://github.com/sp4cerat/Fast-Quadric-Mesh-Simplification
        Indices::U32(self.indices.clone())
    }

    pub fn colors_from_biomes(&self) -> Vec<[f32; 4]> {
        self.biomes
            .iter()
            .map(|b| b.color().as_rgba_f32())
            .collect()
    }
}
