use bevy::prelude::*;
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use lerp::{num_traits::Pow, Lerp};
use noise::{NoiseFn, OpenSimplex, Perlin};

use crate::mesh::{NoiseValues, TerrainMesh};

use self::smoothness::SmoothFunction;

pub mod smoothness;

#[derive(Reflect, Debug, Clone, Resource, InspectorOptions)]
#[reflect(Resource, Default, InspectorOptions)]
pub struct TerrainMap {
    #[inspector(min = 0)]
    seed: u32,
    #[inspector(min = 0.01)]
    mesh_smoothness: f64,
    mesh_smoothness_type: SmoothFunction,
    #[inspector(min = 10)]
    grid_size: u32,
    #[inspector(min = 1.)]
    cell_size: f32,
    #[inspector(min = 0.01, max = 10.0)]
    terrain_frequency: f64,
    #[inspector(min = 0.01, max = 10.0)]
    terrain_scale: f64,
    #[inspector(min = 1, max = 16)]
    terrain_octave: u16,
    #[inspector(min = 1., max = 1000.0)]
    initial_height: f64,
    #[inspector(min = -100., max = 500.)]
    min_terrain_level: f64,
    #[inspector(min = 0., max = 5000.)]
    max_terrain_level: f64,
    #[inspector(min = 0.01, max = 100.0)]
    moisture_scale: f64,
    #[inspector(min = 0.01, max = 100.0)]
    temp_scale: f64,
    #[inspector(min = -273., max = 25.)]
    min_temp: f64,
    #[inspector(min = -5., max = 100.)]
    max_temp: f64,

    #[reflect(ignore)]
    simplex: OpenSimplex,
    #[reflect(ignore)]
    perlin: Perlin,
    #[reflect(ignore)]
    pub has_changes: bool,
}

impl Default for TerrainMap {
    fn default() -> Self {
        Self {
            has_changes: false,
            seed: 0,
            mesh_smoothness: 2.,
            mesh_smoothness_type: SmoothFunction::default(),
            grid_size: 100,
            cell_size: 10f32,
            terrain_frequency: 5.,
            terrain_scale: 1.34,
            terrain_octave: 4,
            initial_height: 1.,
            simplex: OpenSimplex::new(0),
            perlin: Perlin::new(0),
            min_terrain_level: 5.,
            max_terrain_level: 40.,
            moisture_scale: 0.06,
            temp_scale: 0.7,
            min_temp: -70.,
            max_temp: 100.,
        }
    }
}

impl PartialEq for TerrainMap {
    fn eq(&self, other: &Self) -> bool {
        self.seed == other.seed
    }
}

impl TerrainMap {
    pub fn new(seed: u32) -> Self {
        Self {
            has_changes: false,
            seed,
            mesh_smoothness: 2.,
            mesh_smoothness_type: SmoothFunction::default(),
            grid_size: 100,
            cell_size: 10.,
            terrain_frequency: 5.,
            terrain_scale: 1.34,
            terrain_octave: 4,
            initial_height: 1.,
            simplex: OpenSimplex::new(seed),
            perlin: Perlin::new(seed),
            min_terrain_level: 0.,
            max_terrain_level: 20.,
            moisture_scale: 0.06,
            temp_scale: 0.7,
            min_temp: -70.,
            max_temp: 100.,
        }
    }

    pub fn new_parameterized(
        seed: u32,
        grid_size: u32,
        cell_size: f32,
        mesh_smoothness: f64,
    ) -> Self {
        Self {
            has_changes: false,
            seed,
            mesh_smoothness,
            mesh_smoothness_type: SmoothFunction::default(),
            grid_size,
            cell_size,
            terrain_frequency: 5.,
            terrain_scale: 1.34,
            terrain_octave: 4,
            initial_height: 1.,
            simplex: OpenSimplex::new(seed),
            perlin: Perlin::new(seed),
            min_terrain_level: 0.,
            max_terrain_level: 20.,
            moisture_scale: 0.06,
            temp_scale: 0.7,
            min_temp: -70.,
            max_temp: 100.,
        }
    }

    pub fn update_seed(&mut self) {
        let seed = self.seed;
        self.perlin = Perlin::new(seed);
        self.simplex = OpenSimplex::new(seed);
    }

    pub fn terrain_mesh(&self) -> TerrainMesh {
        let vertex_offset = self.cell_size * 0.5;
        let mut vertices =
            Vec::with_capacity((self.grid_size as usize + 1) * (self.grid_size as usize + 1));
        let mut indices = Vec::with_capacity(self.grid_size as usize * self.grid_size as usize * 6);
        let mut colors =
            Vec::with_capacity((self.grid_size as usize + 1) * (self.grid_size as usize + 1));
        let mut normals =
            Vec::with_capacity((self.grid_size as usize + 1) * (self.grid_size as usize + 1) * 3);

        let mut v = 0usize;
        for x in 0..=self.grid_size {
            for y in 0..=self.grid_size {
                let x_f64 = x as f64;
                let y_f64 = y as f64;

                let height = self.get_height(x_f64, y_f64);
                let smoothed_height = if self.mesh_smoothness_type == SmoothFunction::Identity {
                    height
                } else {
                    self.mesh_smoothness_type
                        .get_smoothed(height, self.mesh_smoothness)
                };
                let moisture = self.get_moisture(x_f64, y_f64);
                let temperature = self.get_temperature(x_f64, y_f64);
                vertices.push(Vec3::new(
                    x as f32 * self.cell_size - vertex_offset,
                    self.min_terrain_level
                        .lerp(self.max_terrain_level, smoothed_height) as f32,
                    y as f32 * self.cell_size - vertex_offset,
                ));
                colors.push(NoiseValues {
                    temperature,
                    height,
                    moisture,
                });

                v += 1;
            }
        }

        v = 0usize;
        for _x in 0..self.grid_size {
            for _y in 0..self.grid_size {
                indices.push(v as u32);
                indices.push(v as u32 + 1);
                indices.push(v as u32 + self.grid_size + 1);
                indices.push(v as u32 + self.grid_size + 1);
                indices.push(v as u32 + 1);
                indices.push(v as u32 + self.grid_size + 2);

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

            normals.push(face_normal);
        }

        TerrainMesh::new(vertices, indices, colors, normals)
    }

    fn get_height(&self, x: f64, y: f64) -> f64 {
        let mut result = 0.0;
        let mut scale = self.terrain_scale;
        let mut height = self.initial_height;
        let mut height_acc = 0.;

        for _ in 0..self.terrain_octave {
            result += (self.perlin.get([x * scale, y * scale]) * 0.5 + 0.5) * height;
            height_acc += height;
            scale *= 2.0;
            height *= 0.5;
        }

        result /= height_acc;

        result.pow(self.terrain_frequency)
    }

    fn get_temperature(&self, x: f64, y: f64) -> f64 {
        self.simplex.get([x * self.temp_scale, y * self.temp_scale]) * 0.5 + 0.5
    }

    fn get_moisture(&self, x: f64, y: f64) -> f64 {
        self.simplex
            .get([x * self.moisture_scale, y * self.moisture_scale])
            * 0.5
            + 0.5
    }
}
