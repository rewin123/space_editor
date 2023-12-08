use bevy::{prelude::*, render::mesh::Indices};

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
    pub ambient_values: Vec<NoiseValues>,
    pub normals: Vec<Vec3>,
}

impl TerrainMesh {
    pub fn new(
        vertices: Vec<Vec3>,
        indices: Vec<u32>,
        ambient_values: Vec<NoiseValues>,
        normals: Vec<Vec3>,
    ) -> Self {
        Self {
            vertices,
            indices,
            ambient_values,
            normals,
        }
    }

    pub fn indices(&self) -> Indices {
        Indices::U32(self.indices.clone())
    }

    pub fn colors_from_noise(&self) -> Vec<[f32; 4]> {
        self.ambient_values
            .iter()
            .map(|n| {
                [
                    n.temperature as f32 / 1.1,
                    n.height as f32 + 0.6,
                    n.moisture as f32,
                    1.0,
                ]
            })
            .collect()
    }
}
