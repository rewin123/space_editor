
use bevy::prelude::*;
use bevy_xpbd_3d::prelude::*;

use super::RigidBodyPrefab;

pub type Vector = bevy_xpbd_3d::math::Vector;
pub type Scalar = bevy_xpbd_3d::math::Scalar;

#[derive(Debug, Component, Reflect, Clone)]
#[reflect(Component)]
pub enum ColliderPrefab {
    Cuboid(Vector)
}

impl Default for ColliderPrefab {
    fn default() -> Self {
        ColliderPrefab::Cuboid(Vector::new(1.0, 1.0, 1.0))
    }
}

impl ColliderPrefab {
    pub fn to_collider(&self) -> Collider {
        match self {
            ColliderPrefab::Cuboid(bbox) => {
                Collider::cuboid(bbox.x, bbox.y, bbox.z)
            },
        }
    }
}

pub fn update_collider(
    mut commands : Commands,
    query : Query<(Entity, &ColliderPrefab, Option<&RigidBodyPrefab>, Option<&Transform>), Changed<ColliderPrefab>>
) {
    for (e, collider, rigidbody, transform) in query.iter() {
        commands.entity(e).remove::<Collider>();
        commands.entity(e).insert(collider.to_collider());

        if rigidbody.is_none() {
            commands.entity(e).insert(RigidBodyPrefab::Static);
        }

        if transform.is_none() {
            commands.entity(e).insert(TransformBundle::default());
        }
    }
}

// pub fn debug_draw_collider(
//     mut gizmo : Gizmos,
//     query : Query<(Entity, &Collider), Changed<ColliderPrefab>>
// ) {

// }