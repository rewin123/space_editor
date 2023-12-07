use bevy::{prelude::*, render::mesh::Indices};
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use lerp::{num_traits::Pow, Lerp};
use noise::{NoiseFn, OpenSimplex, Perlin};

#[derive(Debug, Clone)]
pub struct NoiseValues {
    pub height: f64,
    pub moisture: f64,
    pub temperature: f64,
}

#[derive(Reflect, Debug, Clone, Resource, InspectorOptions)]
#[reflect(Resource, Default, InspectorOptions)]
pub struct TerrainMap {
    #[inspector(min = 0)]
    seed: u32,
    #[inspector(min = 10)]
    size: [u32; 2],
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
            size: [100; 2],
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
            size: [100; 2],
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

    pub fn new_sized(seed: u32, size: [u32; 2], cell_size: f32) -> Self {
        Self {
            has_changes: false,
            seed,
            size,
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

    pub fn terrain_mesh(&self) -> (Vec<Vec3>, Indices, Vec<[f32; 4]>) {
        let vertex_offset = self.cell_size * 0.5;

        let mut vertices =
            Vec::with_capacity((self.size[0] as usize + 1) * (self.size[1] as usize + 1));
        let mut indices = Vec::with_capacity(self.size[0] as usize * self.size[1] as usize * 6);
        let mut colors =
            Vec::with_capacity((self.size[0] as usize + 1) * (self.size[1] as usize + 1));

        let mut v = 0usize;

        for x in 0..=self.size[0] {
            for y in 0..=self.size[1] {
                let x_f64 = x as f64;
                let y_f64 = y as f64;

                let height = self.get_height(x_f64, y_f64);
                let moisture = self.get_moisture(x_f64, y_f64);
                let temperature = self.get_temperature(x_f64, y_f64);
                vertices.push(Vec3::new(
                    x as f32 * self.cell_size - vertex_offset,
                    self.min_terrain_level.lerp(self.max_terrain_level, height) as f32,
                    y as f32 * self.cell_size - vertex_offset,
                ));
                colors.push([temperature as f32, height as f32, moisture as f32, 1.0]);

                v += 1;
            }
        }

        v = 0;
        for _x in 0..self.size[0] {
            for _y in 0..self.size[1] {
                indices.push(v as u32);
                indices.push(v as u32 + 1);
                indices.push(v as u32 + self.size[0] + 1);
                indices.push(v as u32 + self.size[1] + 1);
                indices.push(v as u32 + 1);
                indices.push(v as u32 + self.size[1] + 2);

                v += 1;
            }
            v += 1;
        }

        (vertices, Indices::U32(indices), colors)
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
