use bevy::prelude::*;

use crate::prelude::EditorRegistryExt;


pub type TileId = u32;

pub struct TileMapPlugin;

impl Plugin for TileMapPlugin {
    fn build(&self, app: &mut App) {
        app.editor_registry::<TileMap>();
        app.register_type::<Tile>();
        app.register_type::<TileId>();
        app.register_type::<UVec3>();
    }
}

#[derive(Debug, Clone, Reflect, Default, Component)]
#[reflect(Default, Component)]
pub struct TileMap {
    pub tiles: Vec<Tile>,
}

#[derive(Debug, Clone, Reflect, Default)]
#[reflect(Default)]
pub struct Tile {
    pub id : TileId,
    pub scene_path : String,
    pub name : String,
    pub bbox : UVec3,
    pub origin : Option<Vec3>
}

#[derive(Debug, Clone, Reflect, Component)]
#[reflect(Component)]
pub struct TileComponent {
    pub id : TileId,
    pub rotation : IVec3,
    pub map : Entity,
    pub tile : Entity,
}