use bevy::prelude::*;
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};
use lerp::num_traits::Pow;
use noise::{NoiseFn, OpenSimplex, Perlin, SuperSimplex};

mod smoothness;

pub use smoothness::*;

pub type GridValues = Vec<(u32, u32, NoiseValues)>;
pub trait Grid {
    fn grid(&self) -> &GridValues;
    fn grid_mut(&mut self) -> &mut GridValues;
}

impl Grid for HeightMap {
    fn grid(&self) -> &GridValues {
        &self.grid
    }

    fn grid_mut(&mut self) -> &mut GridValues {
        &mut self.grid
    }
}


#[derive(Reflect, Default, Debug, Clone)]
pub enum NoiseAlgorithm {
    #[default]
    Perlin,
    OpenSimplex,
    SuperSimplex,
}

#[derive(Reflect, Default, Debug, Clone)]
pub struct NoiseValues {
    pub height: f64,
    pub smoothed_height: Option<f64>,
    pub moisture: f64,
    pub temperature: f64,
}

#[derive(Component, Reflect, Debug, Clone, Default, InspectorOptions)]
#[reflect(Component, Default, InspectorOptions)]
pub struct HeightMap {
    pub grid: GridValues,
}

impl HeightMap {
    pub fn is_empty(&self) -> bool {
        self.grid.is_empty()
    }
}

#[derive(Component, Debug, Clone, InspectorOptions, Reflect)]
#[reflect(Component, Default, InspectorOptions)]
pub struct MapSettings {
    #[inspector(min = 0)]
    seed: u32,
    #[inspector(min = 0.01)]
    mesh_smoothness: f64,
    mesh_smoothness_type: SmoothFunction,
    elevation_noise: NoiseAlgorithm,
    moisture_noise: NoiseAlgorithm,
    temperature_noise: NoiseAlgorithm,
    #[inspector(min = 5, max = 528)]
    pub grid_size: u32,
    #[inspector(min = 1.)]
    pub cell_size: f32,
    #[inspector(min = 0.01, max = 10.0)]
    terrain_frequency: f64,
    #[inspector(min = 0.01, max = 10.0)]
    terrain_scale: f64,
    #[inspector(min = 1, max = 16)]
    terrain_octave: u16,
    #[inspector(min = 0.1, max = 1000.0)]
    initial_height: f64,
    #[inspector(min = -100., max = 500.)]
    pub min_terrain_level: f64,
    #[inspector(min = 0., max = 5000.)]
    pub max_terrain_level: f64,
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
    super_simplex: SuperSimplex,
    #[reflect(ignore)]
    perlin: Perlin,
    #[reflect(ignore)]
    pub has_changes: bool,
}

impl Default for MapSettings {
    fn default() -> Self {
        Self {
            has_changes: false,
            seed: 0,
            mesh_smoothness: 2.,
            mesh_smoothness_type: SmoothFunction::default(),
            grid_size: 100,
            cell_size: 10.,
            terrain_frequency: 5.,
            terrain_scale: 1.34,
            terrain_octave: 4,
            initial_height: 1.,
            simplex: OpenSimplex::new(0),
            super_simplex: SuperSimplex::new(0),
            perlin: Perlin::new(0),
            min_terrain_level: 5.,
            max_terrain_level: 40.,
            moisture_scale: 0.06,
            temp_scale: 0.7,
            min_temp: -70.,
            max_temp: 100.,
            elevation_noise: NoiseAlgorithm::SuperSimplex,
            moisture_noise: NoiseAlgorithm::OpenSimplex,
            temperature_noise: NoiseAlgorithm::Perlin,
        }
    }
}

impl PartialEq for MapSettings {
    fn eq(&self, other: &Self) -> bool {
        self.seed == other.seed
    }
}

impl MapSettings {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            simplex: OpenSimplex::new(seed),
            perlin: Perlin::new(seed),
            ..default()
        }
    }

    pub fn new_parameterized(
        seed: u32,
        grid_size: u32,
        cell_size: f32,
        mesh_smoothness: f64,
    ) -> Self {
        Self {
            seed,
            mesh_smoothness,
            grid_size,
            cell_size,
            simplex: OpenSimplex::new(seed),
            perlin: Perlin::new(seed),
            ..default()
        }
    }

    pub fn update_seed(&mut self) {
        let seed = self.seed;
        self.perlin = Perlin::new(seed);
        self.simplex = OpenSimplex::new(seed);
        self.super_simplex = SuperSimplex::new(seed);
    }

    pub fn heightmap(&self) -> HeightMap {
        let mut map = HeightMap::default();

        for x in 0..=self.grid_size {
            for y in 0..=self.grid_size {
                let x_f64 = f64::from(x);
                let y_f64 = f64::from(y);

                let height = self.get_height(x_f64, y_f64);
                let smoothed_height = if self.mesh_smoothness_type == SmoothFunction::Identity {
                    None
                } else {
                    Some(
                        self.mesh_smoothness_type
                            .get_smoothed(height, self.mesh_smoothness),
                    )
                };
                let moisture = self.get_moisture(x_f64, y_f64);
                let temperature = self.get_temperature(x_f64, y_f64);

                let values = NoiseValues {
                    height,
                    smoothed_height,
                    moisture,
                    temperature,
                };
                map.grid.push((x, y, values));
            }
        }

        map
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
