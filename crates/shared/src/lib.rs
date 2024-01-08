use bevy::prelude::*;

pub mod prelude {
    pub use crate::{
        EditorCameraMarker, EditorEvent, EditorPrefabPath, EditorSet, EditorState, PrefabMarker,
        PrefabMemoryCache,
    };
}

/// Component Marker to display entity in Editor
#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct PrefabMarker;

/// Component marker that manages editor only camera
/// A camera tagged with this component will not be in use during playmode
#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct EditorCameraMarker;

/// Editor states (`Editor`, `GamePrepare`, `Game`)
#[derive(States, Default, Debug, Clone, Hash, Eq, PartialEq)]
pub enum EditorState {
    /// Diplays Editor / Editor mode
    Editor,
    /// Editor is loading the game
    GamePrepare,
    /// Play mode, game is being executed
    #[default]
    Game,
}

/// Sets for separate game and editor logic
#[derive(SystemSet, Hash, Eq, PartialEq, Clone, Debug)]
pub enum EditorSet {
    /// Editor mode System Set
    Editor,
    /// Play mode System Set
    Game,
}

#[derive(Resource, Default)]
pub struct PrefabMemoryCache {
    pub scene: Option<Handle<DynamicScene>>,
}

#[derive(Clone, Debug)]
/// How/Where porefab data is stored
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
