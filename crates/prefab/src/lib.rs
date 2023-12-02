// Both will be deprecated soon
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

#[cfg(all(feature = "f32", feature = "f64"))]
compile_error!("feature \"f32\" and feature \"f64\" cannot be enabled at the same time");

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

/// Module contains custom registry options to store clone functions and bundles in UI
pub mod editor_registry;

use bevy::prelude::*;

use crate::plugins::PrefabPlugin;
use shared::{EditorSet, EditorState};

/// Public usage of packages that used in this crate
pub mod ext {
    pub use bevy::prelude::*;
    pub use bevy_debug_grid;
    pub use bevy_egui::*;
    pub use bevy_inspector_egui::prelude::*;
    pub use bevy_mod_picking::prelude::*;
    pub use bevy_panorbit_camera::*;
}

/// All useful structure from this crate
pub mod prelude {
    pub use crate::editor_registry::*;
    pub use crate::load::PrefabBundle;
    pub use crate::plugins::*;
    pub use crate::SpaceEditorPlugin;
    pub use crate::*;
    pub use shared::PrefabMarker;
}

/// Plugin to activate editor UI and prefab plugin
#[derive(Default)]
pub struct SpaceEditorPlugin {}

impl Plugin for SpaceEditorPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<PrefabPlugin>() {
            app.add_plugins(PrefabPlugin);
        }

        app.configure_sets(
            PreUpdate,
            EditorSet::Game.run_if(in_state(EditorState::Game)),
        );
        app.configure_sets(Update, EditorSet::Game.run_if(in_state(EditorState::Game)));
        app.configure_sets(
            PostUpdate,
            EditorSet::Game.run_if(in_state(EditorState::Game)),
        );

        app.configure_sets(
            PreUpdate,
            EditorSet::Editor.run_if(in_state(EditorState::Editor)),
        );
        app.configure_sets(
            Update,
            EditorSet::Editor.run_if(in_state(EditorState::Editor)),
        );
        app.configure_sets(
            PostUpdate,
            EditorSet::Editor.run_if(in_state(EditorState::Editor)),
        );

        app.configure_sets(
            Update,
            (
                PrefabSet::PrefabLoad,
                PrefabSet::Relation,
                PrefabSet::RelationApply,
                PrefabSet::DetectPrefabChange,
                PrefabSet::PrefabChangeApply,
            )
                .chain(),
        );

        app.add_systems(Update, apply_deferred.in_set(PrefabSet::RelationApply));
        app.add_systems(Update, apply_deferred.in_set(PrefabSet::PrefabChangeApply));
    }
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
