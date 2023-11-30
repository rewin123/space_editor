// Both will be deprecated soon
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

#[cfg(all(feature = "f32", feature = "f64"))]
compile_error!("feature \"f32\" and feature \"f64\" cannot be enabled at the same time");

/// Module contains all prefab logic and components
pub mod prefab;

/// Module contains custom registry options to store clone functions and bundles in UI
pub mod editor_registry;

use bevy::prelude::*;

#[cfg(feature = "bevy_xpbd_3d")]
use optional::OptionalPlugin;
use prefab::PrefabPlugin;
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
    pub use super::editor_registry::*;
    pub use super::prefab::load::PrefabBundle;
    pub use super::prefab::*;
    pub use super::SpaceEditorPlugin;
    pub use super::*;
    pub use shared::PrefabMarker;

    #[cfg(feature = "bevy_xpbd_3d")]
    pub use super::optional::bevy_xpbd_plugin::*;
}

/// Plugin to activate editor UI and prefab plugin
#[derive(Default)]
pub struct SpaceEditorPlugin {}

impl Plugin for SpaceEditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PrefabPlugin);

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
