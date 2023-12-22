#![allow(clippy::too_many_arguments)]
use bevy::prelude::*;
use heightmap::{HeightMap, MapSettings, SmoothFunction};
use mesh::TerrainDrawTag;
use space_editor_ui::ui_registration::EditorUiExt;
use space_prefab::editor_registry::EditorRegistryExt;

pub mod heightmap;
#[cfg(feature = "space_editor")]
pub mod inspector;
#[cfg(feature = "meshed")]
pub mod mesh;

pub use inspector::TerraingenInspectorPlugin;
use space_shared::PrefabMarker;

const TERRAIN_BUNDLE_CATEGORY: &str = "Terrain";

#[derive(Debug, Default)]
pub struct TerraingenPlugin;

impl Plugin for TerraingenPlugin {
    fn build(&self, app: &mut App) {
        app.editor_registry::<MapSettings>()
            .editor_registry::<HeightMap>()
            .register_type::<SmoothFunction>()
            .editor_registry::<TerrainDrawTag>();
        app.add_event::<UpdateTerrain>();
        app.editor_bundle(
            TERRAIN_BUNDLE_CATEGORY,
            "Square terrain",
            TerrainBundle::default(),
        );
        app.add_systems(PostUpdate, update_spawned_terrain);
    }
}

#[derive(Event, Clone)]
pub enum UpdateTerrain {
    All,
    One(Entity),
}

#[derive(Bundle, Default, Clone)]
pub struct TerrainBundle {
    pub terrain: Terrain,
    pub settings: MapSettings,
    pub heightmap: HeightMap,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
    pub prefab_marker: PrefabMarker,
}

#[derive(Component, Default, Clone)]
pub struct Terrain;

#[derive(Component)]
pub struct TerrainChunk;

//Draw spawned terrain
pub fn update_spawned_terrain(
    mut update_events: EventWriter<UpdateTerrain>,
    mut query: Query<Entity, Added<Terrain>>,
) {
    for entity in query.iter_mut() {
        update_events.send(UpdateTerrain::One(entity));
    }
}
