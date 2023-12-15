use bevy::prelude::*;
use biomes::Biomes;

pub mod biomes;
mod resources;
pub mod systems;

use prefab::editor_registry::EditorRegistryExt;
use resources::smoothness::SmoothFunction;
pub use resources::TerrainMap;
use systems::TerrainDrawTag;

#[derive(Debug, Default)]
pub struct TerraingenPlugin;

impl Plugin for TerraingenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainMap>()
            .register_type::<TerrainMap>()
            .register_type::<SmoothFunction>()
            .register_type::<Biomes>()
            .editor_registry::<TerrainDrawTag>();
    }
}
