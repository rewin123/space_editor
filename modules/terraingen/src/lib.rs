use bevy::{utils::HashMap, prelude::*};
use bevy_inspector_egui::InspectorOptions;
use noise::{
    core::perlin::perlin_2d,
    permutationtable::PermutationTable,
    utils::*,
};
#[derive(Reflect, Debug, Clone, Resource, InspectorOptions)]
#[reflect(Resource, Default)]
pub struct HeightMap {
    seed: u32,
    #[reflect(ignore)]
    hasher: PermutationTable,
}

impl Default for HeightMap {
    fn default() -> Self {
        Self { seed: 0, hasher: PermutationTable::new(0) }
    }
}

impl PartialEq for HeightMap {
    fn eq(&self, other: &Self) -> bool {
        self.seed == other.seed
    }
}

impl HeightMap {
    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            hasher: PermutationTable::new(seed)
        }
    }

    pub fn height_map(&self, size: [u32; 2]) -> HashMap<[u32;2], f64> {
        let hasher = PermutationTable::new(self.seed);
        let mut map = HashMap::new();

        let built_map = PlaneMapBuilder::new_fn(|point, hasher| perlin_2d(point.into(), hasher), &hasher)
            .set_size(size[0] as usize, size[1] as usize)
            .set_x_bounds(-5.0, 5.0)
            .set_y_bounds(-5.0, 5.0)
            .build();

        map
    }
}

#[test]
fn test() {
    let hm = HeightMap::new(0);

    hm.height_map([10, 10]);
}