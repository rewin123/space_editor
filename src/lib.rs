pub mod editor;
pub mod prefab;
pub mod editor_registry;

pub mod optional;

use bevy::{prelude::*, pbr::CascadeShadowConfigBuilder};

use editor::EditorPlugin;
use optional::OptionalPlugin;
use prefab::PrefabPlugin;

pub mod ext {
    pub use bevy_mod_picking::prelude::*;
    pub use bevy_inspector_egui::prelude::*;
    pub use bevy_egui::*;
    pub use bevy::prelude::*;
}

pub mod prelude {
    pub use super::editor::prelude::*;
    pub use super::prefab::*;
    pub use super::SpaceEditorPlugin;
    pub use super::PrefabMarker;
    pub use super::editor_registry::*;
    pub use super::*;

    #[cfg(feature = "bevy_xpbd_3d")]
    pub use super::optional::bevy_xpbd_plugin::*;
}

pub struct SpaceEditorPlugin {

}


impl Default for SpaceEditorPlugin {
    fn default() -> Self {
        Self {

        }
    }
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


//editor shows only entities with this marker
#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct PrefabMarker;

#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct EditorCameraMarker;


#[derive(States, Default, Debug, Clone, Hash, Eq, PartialEq)]
pub enum EditorState {
    Editor, // editor is showing
    GamePrepare, //editor preparing to run game
    #[default]
    Game // playing game
}

#[derive(SystemSet, Hash, Eq, PartialEq, Clone, Debug)]
pub enum EditorSet {
    Editor,
    Game
}

#[derive(SystemSet, Hash, Eq, PartialEq, Clone, Debug)]
pub enum PrefabSet {
    PrefabLoad,
    Relation,
    RelationApply,
    DetectPrefabChange,
    PrefabChangeApply
}

pub fn simple_editor_setup(mut commands: Commands) {
    commands.insert_resource(bevy::pbr::DirectionalLightShadowMap { size: 4096 });
   // light
    commands.spawn(DirectionalLightBundle {
        directional_light : DirectionalLight { shadows_enabled: true, ..default() },
        transform: Transform::from_xyz(4.0, 8.0, 4.0).with_rotation(Quat::from_rotation_x(-std::f32::consts::PI / 4.)),
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