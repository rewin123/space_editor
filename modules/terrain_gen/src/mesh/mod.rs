use bevy::{prelude::*, render::mesh::Indices};
use lerp::Lerp;

use crate::heightmap::{Grid, HeightMap, MapSettings};

pub mod systems;

pub trait Generation {
    type Item: Grid;
    fn generate_mesh(grid: &Self::Item, settings: &MapSettings) -> TerrainMesh;
}

#[derive(Resource, Clone, Debug, Default)]
pub struct TerrainMeshId(AssetId<Mesh>);

#[derive(Component, Default, Clone, Reflect)]
pub struct TerrainDrawTag;

#[derive(Reflect, Default, Debug, Clone, Resource)]
#[reflect(Resource, Default)]
pub struct TerrainMesh {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub colors: Vec<Color>,
    pub normals: Vec<Vec3>,
}

impl TerrainMesh {
    pub fn new(
        vertices: Vec<Vec3>,
        indices: Vec<u32>,
        colors: Vec<Color>,
        normals: Vec<Vec3>,
    ) -> Self {
        Self {
            vertices,
            indices,
            colors,
            normals,
        }
    }

    pub fn indices(&self) -> Indices {
        Indices::U32(self.indices.clone())
    }

    pub fn colors(&self) -> Vec<[f32; 4]> {
        self.colors
            .iter()
            .map(|c| [c.r(), c.g(), c.b(), c.a()])
            .collect()
    }

    pub fn simplify(&mut self) {
        // TODO: Optimize meshes/mesh count
        // https://github.com/sp4cerat/Fast-Quadric-Mesh-Simplification
        todo!()
    }
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

fn terrain_colormap(temperature: f32, height: f32, moisture: f32) -> Vec3 {
    // Temperature colormap
    let temperature_color = if temperature < 0.0 {
        Vec3 {
            x: 0.0,
            y: 0.0,
            z: clamp(0.5 - temperature.abs(), 0.0, 1.0),
        }
    } else {
        Vec3 {
            x: clamp(temperature, 0.0, 1.0),
            y: 0.0,
            z: 0.0,
        }
    };

    // Height colormap
    let height_color = Vec3 {
        x: clamp(height, 0.0, 1.0),
        y: 0.5,
        z: 0.0,
    };

    // Moisture colormap
    let moisture_color = Vec3 {
        x: 0.0,
        y: clamp(moisture, 0.0, 1.0),
        z: 0.0,
    };

    // Combine colors based on factors (you may adjust these weights)
    let final_color = Vec3 {
        x: 0.4 * temperature_color.x + 0.4 * height_color.x + 0.2 * moisture_color.x,
        y: 0.4 * temperature_color.y + 0.4 * height_color.y + 0.2 * moisture_color.y,
        z: 0.4 * temperature_color.z + 0.4 * height_color.z + 0.2 * moisture_color.z,
    };

    final_color
}

impl Generation for TerrainMesh {
    type Item = HeightMap;
    fn generate_mesh(grid: &Self::Item, settings: &MapSettings) -> TerrainMesh {
        let heightmap = grid.grid().clone();
        let vertex_offset = settings.cell_size * 0.5;
        let mut vertices = Vec::with_capacity(
            (settings.grid_size as usize + 1) * (settings.grid_size as usize + 1),
        );
        let mut indices =
            Vec::with_capacity(settings.grid_size as usize * settings.grid_size as usize * 6);
        let mut colors = Vec::with_capacity(
            (settings.grid_size as usize + 1) * (settings.grid_size as usize + 1),
        );
        let mut normals = vec![Vec3::ZERO; heightmap.len()];

        for (x, y, value) in heightmap {
            let height = if let Some(height) = value.smoothed_height {
                height
            } else {
                value.height
            };
            vertices.push(Vec3::new(
                x as f32 * settings.cell_size - vertex_offset,
                settings
                    .min_terrain_level
                    .lerp(settings.max_terrain_level, height) as f32,
                y as f32 * settings.cell_size - vertex_offset,
            ));

            let terr_color = terrain_colormap(
                value.temperature as f32,
                height as f32,
                value.moisture as f32,
            );
            colors.push(Color::Rgba {
                red: terr_color.x,
                green: terr_color.y,
                blue: terr_color.z,
                alpha: 1.0,
            });
        }

        let mut v = 0usize;
        for _x in 0..settings.grid_size {
            for _y in 0..settings.grid_size {
                indices.push(v as u32);
                indices.push(v as u32 + 1);
                indices.push(v as u32 + settings.grid_size + 1);
                indices.push(v as u32 + settings.grid_size + 1);
                indices.push(v as u32 + 1);
                indices.push(v as u32 + settings.grid_size + 2);

                v += 1;
            }
            v += 1;
        }

        for index in indices.chunks_exact(3) {
            let i_0 = index[0] as usize;
            let i_1 = index[1] as usize;
            let i_2 = index[2] as usize;

            let v1 = vertices[i_1] - vertices[i_0];
            let v2 = vertices[i_2] - vertices[i_0];
            let face_normal = v1.cross(v2).normalize();

            normals[i_0] += face_normal;
            normals[i_1] += face_normal;
            normals[i_2] += face_normal;
        }

        for normal in normals.iter_mut() {
            *normal = normal.normalize();
        }

        TerrainMesh {
            vertices,
            indices,
            colors,
            normals,
        }
    }
}
