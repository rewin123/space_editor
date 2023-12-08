use bevy::prelude::*;

pub mod mesh;
mod resources;
pub mod systems;

pub use resources::TerrainMap;

#[derive(Debug, Default)]
pub struct TerraingenPlugin;

impl Plugin for TerraingenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TerrainMap>()
            .register_type::<TerrainMap>();
    }
}
