use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;
use space_editor_ui::{
    prelude::{EditorRegistryExt, EditorState, PrefabSet},
    settings::RegisterSettingsBlockExt,
};

use crate::{
    collider::{self, ColliderPart, ColliderPrefabCompound, ColliderPrimitive},
    spatial_query::register_xpbd_spatial_types,
};

pub type Vector = bevy_xpbd_3d::math::Vector;
pub type Scalar = bevy_xpbd_3d::math::Scalar;

pub struct BevyXpbdPlugin;

impl Plugin for BevyXpbdPlugin {
    fn build(&self, app: &mut App) {
        println!("BevyXpbdPlugin::build");
        app.add_plugins(PhysicsPlugins::default());
        app.add_plugins(bevy_xpbd_3d::plugins::PhysicsDebugPlugin::default());

        app.editor_registry::<collider::ColliderPrefab>()
            .editor_registry::<RigidBodyPrefab>()
            .editor_registry::<Mass>()
            .editor_registry::<Friction>()
            .editor_registry::<Restitution>()
            .editor_registry::<LinearDamping>()
            .editor_registry::<AngularDamping>()
            .editor_registry::<Inertia>()
            .editor_registry::<CenterOfMass>()
            .editor_registry::<LockedAxes>()
            .editor_registry::<GravityScale>()
            .editor_registry::<Sensor>();

        app.register_type::<ColliderPrimitive>()
            .register_type::<ColliderPart>()
            .register_type::<Vec<ColliderPart>>()
            .register_type::<ColliderPrefabCompound>();

        register_xpbd_spatial_types(app);

        app.add_systems(
            PreUpdate,
            (editor_pos_change)
                .in_set(PrefabSet::DetectPrefabChange)
                .run_if(in_state(EditorState::Editor)),
        );

        app.add_systems(
            Update,
            (collider::update_collider).in_set(PrefabSet::DetectPrefabChange),
        );

        app.add_systems(
            Update,
            rigidbody_type_change_in_editor
                .run_if(in_state(EditorState::Editor))
                .in_set(PrefabSet::DetectPrefabChange),
        );
        app.add_systems(
            Update,
            rigidbody_type_change
                .run_if(in_state(EditorState::Game))
                .in_set(PrefabSet::DetectPrefabChange),
        );
        app.add_systems(
            OnEnter(EditorState::Editor),
            force_rigidbody_type_change_in_editor,
        );
        app.add_systems(OnEnter(EditorState::Game), force_rigidbody_type_change);
        app.add_systems(
            Update,
            (sync_position_spawn).run_if(in_state(EditorState::Editor)),
        );

        if app.is_plugin_added::<space_editor_ui::ui_plugin::EditorUiCore>() {
            app.register_settings_block("Bevy XPBD 3D", |ui, _, world| {
                ui.checkbox(
                    &mut world
                        .resource_mut::<bevy_xpbd_3d::prelude::PhysicsDebugConfig>()
                        .enabled,
                    "Show bevy xpbd debug render",
                );
                ui.checkbox(
                    &mut world
                        .resource_mut::<bevy_xpbd_3d::prelude::PhysicsDebugConfig>()
                        .hide_meshes,
                    "Hide debug meshes",
                );
            });
        }
    }
}

fn sync_position_spawn(
    mut commands: Commands,
    query: Query<
        (Entity, &Transform),
        Or<(Changed<RigidBodyPrefab>, Changed<collider::ColliderPrefab>)>,
    >,
) {
    for (e, tr) in query.iter() {
        commands
            .entity(e)
            .insert((Position(tr.translation), Rotation(tr.rotation)));
    }
}

#[derive(Component, Clone, Default, Reflect)]
#[reflect(Component)]
/// Available bevy_xpbd::RigidBody wrappers
pub enum RigidBodyPrefab {
    Dynamic,
    #[default]
    Static,
    Kinematic,
}

impl RigidBodyPrefab {
    pub const fn to_rigidbody(&self) -> RigidBody {
        match self {
            Self::Dynamic => RigidBody::Dynamic,
            Self::Static => RigidBody::Static,
            Self::Kinematic => RigidBody::Kinematic,
        }
    }

    pub const fn to_rigidbody_editor(&self) -> RigidBody {
        RigidBody::Static
    }
}

fn force_rigidbody_type_change_in_editor(
    mut commands: Commands,
    query: Query<(Entity, &RigidBodyPrefab, Option<&Transform>)>,
) {
    for (e, tp, transform) in query.iter() {
        commands.entity(e).insert(tp.to_rigidbody_editor());
        if let Some(tr) = transform {
            commands
                .entity(e)
                .insert((Position(tr.translation), Rotation(tr.rotation)));
        }
    }
}

fn rigidbody_type_change_in_editor(
    mut commands: Commands,
    query: Query<(Entity, &RigidBodyPrefab, Option<&Transform>), Changed<RigidBodyPrefab>>,
) {
    for (e, tp, transform) in query.iter() {
        info!("Rigidbody type changed in {:?}", e);
        commands
            .entity(e)
            .remove::<RigidBody>()
            .insert(tp.to_rigidbody_editor());
        if let Some(tr) = transform {
            commands
                .entity(e)
                .insert((Position(tr.translation), Rotation(tr.rotation)));
        }
    }
}

fn force_rigidbody_type_change(
    mut commands: Commands,
    query: Query<(Entity, &RigidBodyPrefab, Option<&collider::ColliderPrefab>)>,
) {
    for (e, tp, _col) in query.iter() {
        commands
            .entity(e)
            .remove::<RigidBody>()
            .insert(tp.to_rigidbody());
        // if let Some(col) = col {
        //     commands.entity(e).insert(col.to_collider());
        // }
    }
}

fn rigidbody_type_change(
    mut commands: Commands,
    query: Query<(Entity, &RigidBodyPrefab), Changed<RigidBodyPrefab>>,
) {
    for (e, tp) in query.iter() {
        commands.entity(e).remove::<RigidBody>();
        commands.entity(e).insert(tp.to_rigidbody());
    }
}

pub fn editor_pos_change(
    mut query: Query<(&mut Position, &mut Rotation, &Transform), Changed<Transform>>,
) {
    for (mut pos, mut rot, transform) in query.iter_mut() {
        // let transform = transform.compute_transform();
        if pos.0 != transform.translation {
            pos.0 = transform.translation;
        }
        if rot.0 != transform.rotation {
            rot.0 = transform.rotation;
        }
    }
}
