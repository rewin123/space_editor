#![allow(clippy::too_many_arguments)]
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
        app.register_type::<MapSettings>()
            .register_type::<HeightMap>()
            .register_type::<SmoothFunction>()
            .editor_registry::<TerrainDrawTag>();

        let settings = MapSettings::default();
        let generated_heightmap = settings.heightmap();
        app.insert_resource(settings)
            .insert_resource(generated_heightmap);
    }
}
