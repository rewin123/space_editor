#[cfg(all(feature = "f32", feature = "f64"))]
compile_error!("feature \"f32\" and feature \"f64\" cannot be enabled at the same time");


/// Module contains all editor UI logic and components
pub mod editor;

/// Module contains all prefab logic and components
pub mod prefab;

/// Module contains custom registry options to store clone functions and bundles in UI
pub mod editor_registry;

/// Optional editor extensions (like activate bevy_xpbd support in editor)
pub mod optional;

use bevy::{prelude::*, pbr::CascadeShadowConfigBuilder};

use editor::EditorPlugin;
use optional::OptionalPlugin;
use prefab::PrefabPlugin;

/// Public usage of packages that used in this crate
pub mod ext {
    pub use bevy_mod_picking::prelude::*;
    pub use bevy_inspector_egui::prelude::*;
    pub use bevy_egui::*;
    pub use bevy::prelude::*;
}

/// All useful structure from this crate
pub mod prelude {
    pub use super::editor::prelude::*;
    pub use super::prefab::*;
    pub use super::SpaceEditorPlugin;
    pub use super::PrefabMarker;
    pub use super::editor_registry::*;
    pub use super::*;
    pub use super::prefab::load::PrefabBundle;

    #[cfg(feature = "bevy_xpbd_3d")]
    pub use super::optional::bevy_xpbd_plugin::*;
}

/// Plugin to activate editor UI and prefab plugin
#[derive(Default)]
pub struct SpaceEditorPlugin {

}

#[cfg(feature = "f32")]
pub mod types {
    pub type STransform = bevy::prelude::Transform;
    pub type SGlobalTransform = bevy::prelude::GlobalTransform;
    pub type SQuat = bevy::prelude::Quat;
    pub type Vector = bevy::prelude::Vec3;
    pub type Scalar = f32;
}

#[cfg(feature = "f64")]
pub mod types {
    use bevy::math::{DVec3, DQuat};
    use bevy_transform64::prelude::*;

    pub type STransform = DTransform;
    pub type SGlobalTransform = DGlobalTransform;
    pub type SQuat = DQuat;
    pub type Vector = DVec3;
    pub type Scalar = f64;
}

impl Plugin for SpaceEditorPlugin {    
    fn build(&self, app: &mut App) {
        app.add_plugins(PrefabPlugin);
        app.add_plugins(OptionalPlugin);
        app.add_plugins(EditorPlugin);

        app.configure_sets(Update, (PrefabSet::PrefabLoad, PrefabSet::Relation, PrefabSet::RelationApply, PrefabSet::DetectPrefabChange, PrefabSet::PrefabChangeApply).chain());

        app.add_systems(Update, apply_deferred.in_set(PrefabSet::RelationApply));
        app.add_systems(Update, apply_deferred.in_set(PrefabSet::PrefabChangeApply));
    }
}


/// Editor work only with entities with this marker
#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct PrefabMarker;

/// Marker for editor camera to disable in play mode
#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct EditorCameraMarker;

/// Editor states (Editor, GamePrepare, Game)
#[derive(States, Default, Debug, Clone, Hash, Eq, PartialEq)]
pub enum EditorState {
    Editor, /// editor is showing
    GamePrepare, /// editor preparing to run game
    #[default]
    Game 
}

/// Sets for separate game and editor logic
#[derive(SystemSet, Hash, Eq, PartialEq, Clone, Debug)]
pub enum EditorSet {
    Editor,
    Game
}

/// All prefab logics collected in this sets to allow easy extend prefab logic
#[derive(SystemSet, Hash, Eq, PartialEq, Clone, Debug)]
pub enum PrefabSet {
    PrefabLoad,
    Relation,
    RelationApply,
    DetectPrefabChange,
    PrefabChangeApply
}

/// This method prepare default lights and camera for editor UI. You can create own conditions for your editor and use this method how example
pub fn simple_editor_setup(mut commands: Commands) {
    commands.insert_resource(bevy::pbr::DirectionalLightShadowMap { size: 4096 });
   // light
    commands.spawn(DirectionalLightBundle {
        directional_light : DirectionalLight { shadows_enabled: true, ..default() },
        transform: Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        cascade_shadow_config: CascadeShadowConfigBuilder {
            ..default()
        }.into(),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    }).insert(bevy_panorbit_camera::PanOrbitCamera::default())
    .insert(bevy_mod_picking::prelude::RaycastPickCamera::default())
    .insert(EditorCameraMarker);
}