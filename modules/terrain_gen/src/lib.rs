use bevy::prelude::*;
use heightmap::{HeightMap, MapSettings, SmoothFunction};
use mesh::TerrainDrawTag;
use space_prefab::editor_registry::EditorRegistryExt;

pub mod heightmap;
#[cfg(feature = "space_editor")]
pub mod inspector;
#[cfg(feature = "meshed")]
pub mod mesh;

pub use inspector::TerraingenInspectorPlugin;

#[derive(Debug, Default)]
pub struct TerraingenPlugin;

impl Plugin for TerraingenPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapSettings>()
            .register_type::<MapSettings>()
            .init_resource::<HeightMap>()
            .register_type::<HeightMap>()
            .register_type::<SmoothFunction>()
            .editor_registry::<TerrainDrawTag>();
    }
}
