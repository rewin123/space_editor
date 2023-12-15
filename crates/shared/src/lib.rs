use bevy::prelude::*;

pub mod prelude {
    pub use crate::{
        EditorCameraMarker, EditorEvent, EditorPrefabPath, EditorSet, EditorState, PrefabMarker,
        PrefabMemoryCache,
    };
}

/// Editor work only with entities with this marker
#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct PrefabMarker;

/// Marker for editor camera to disable in play mode
#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct EditorCameraMarker;

/// Editor states (`Editor`, `GamePrepare`, `Game`)
#[derive(States, Default, Debug, Clone, Hash, Eq, PartialEq)]
pub enum EditorState {
    Editor,
    /// editor is showing
    GamePrepare,
    /// editor preparing to run game
    #[default]
    Game,
}

/// Sets for separate game and editor logic
#[derive(SystemSet, Hash, Eq, PartialEq, Clone, Debug)]
pub enum EditorSet {
    Editor,
    Game,
}

#[derive(Resource, Default)]
pub struct PrefabMemoryCache {
    pub scene: Option<Handle<DynamicScene>>,
}

#[derive(Clone, Debug)]
pub enum EditorPrefabPath {
    File(String),
    MemoryCahce,
}

#[derive(Event)]
pub enum EditorEvent {
    Load(EditorPrefabPath),
    Save(EditorPrefabPath),
    LoadGltfAsPrefab(String),
    StartGame,
}
