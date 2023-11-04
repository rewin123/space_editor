use crate::{EditorState, PrefabSet};
use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

pub mod collider;

pub mod spatial_query;

use crate::prelude::EditorRegistryExt;

use self::collider::{ColliderPart, ColliderPrefabCompound, ColliderPrimitive};
use self::spatial_query::register_xpbd_spatial_types;

pub type Vector = bevy_xpbd_3d::math::Vector;
pub type Scalar = bevy_xpbd_3d::math::Scalar;

pub struct BevyXpbdPlugin;

impl Plugin for BevyXpbdPlugin {
    fn build(&self, app: &mut App) {
        // if !app.is_plugin_added::<bevy_xpbd_3d::prelude::D() {

        // }
        app.add_plugins(PhysicsPlugins::default());

        app.editor_registry::<collider::ColliderPrefab>();
        app.editor_registry::<RigidBodyPrefab>();

        app.editor_registry::<Mass>();
        app.editor_registry::<Friction>();
        app.editor_registry::<Restitution>();
        app.editor_registry::<LinearDamping>();
        app.editor_registry::<AngularDamping>();
        app.editor_registry::<Inertia>();
        app.editor_registry::<CenterOfMass>();
        app.editor_registry::<LockedAxes>();
        app.editor_registry::<GravityScale>();
        app.editor_registry::<Sensor>();

        app.register_type::<ColliderPrimitive>();
        app.register_type::<ColliderPart>();
        app.register_type::<Vec<ColliderPart>>();
        app.register_type::<ColliderPrefabCompound>();

        register_xpbd_spatial_types(app);

        app.add_systems(
            Update,
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
        commands.entity(e).insert(Position(tr.translation));
        commands.entity(e).insert(Rotation(tr.rotation));
    }
}

#[derive(Component, Clone, Default, Reflect)]
#[reflect(Component)]
pub enum RigidBodyPrefab {
    Dynamic,
    #[default]
    Static,
    Kinematic,
}

impl RigidBodyPrefab {
    pub fn to_rigidbody(&self) -> RigidBody {
        match self {
            RigidBodyPrefab::Dynamic => RigidBody::Dynamic,
            RigidBodyPrefab::Static => RigidBody::Static,
            RigidBodyPrefab::Kinematic => RigidBody::Kinematic,
        }
    }

    pub fn to_rigidbody_editor(&self) -> RigidBody {
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
            commands.entity(e).insert(Position(tr.translation));
            commands.entity(e).insert(Rotation(tr.rotation));
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
            commands.entity(e).insert(Position(tr.translation));
            commands.entity(e).insert(Rotation(tr.rotation));
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
