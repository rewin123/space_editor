// Both will be deprecated soon
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

/// Contains all component for prefab logic
pub mod component;
/// Contains systems for loading prefab from file
pub mod load;
/// Module contains all prefab plugin extensions
pub mod plugins;
/// Contains systems for saving prefab
pub mod save;
/// Contains systems for spawning prefabs
pub mod spawn_system;

/// Module for saving subscene state (like edit gltf entities)
pub mod sub_scene;

pub mod editor_registry;

use bevy::prelude::*;

use space_shared::EditorState;

/// Public usage of packages that used in this crate
pub mod ext {
    pub use bevy::prelude::*;
}

/// All useful structure from this crate
pub mod prelude {
    pub use crate::component::*;
    pub use crate::editor_registry::*;
    pub use crate::load::PrefabBundle;
    pub use crate::plugins::*;
    pub use crate::save::*;
    pub use crate::sub_scene::*;
    pub use crate::PrefabSet;
    pub use space_shared::PrefabMarker;
}

/// All prefab logics collected in this sets to allow easy extend prefab logic
#[derive(SystemSet, Hash, Eq, PartialEq, Clone, Debug)]
pub enum PrefabSet {
    PrefabLoad,
    Relation,
    RelationApply,
    DetectPrefabChange,
    PrefabChangeApply,
}
