use bevy::prelude::*;
use persistence::AppPersistenceExt;

pub mod mesh;
mod resources;
pub mod systems;

use prefab::editor_registry::EditorRegistryExt;
pub use resources::TerrainMap;
use systems::TerrainDrawTag;

#[derive(Debug, Default)]
pub struct TerraingenPlugin;

impl Plugin for TerraingenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainMap>()
            .persistence_resource::<TerrainMap>()
            .register_type::<TerrainMap>()
            .editor_registry::<TerrainDrawTag>();
    }
}
