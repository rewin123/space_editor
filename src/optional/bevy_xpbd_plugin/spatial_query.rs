use bevy::prelude::*;
use bevy_inspector_egui::*;
use bevy_xpbd_3d::{math::Quaternion, prelude::*};

use crate::prelude::EditorRegistryExt;

use super::{collider::ColliderPrimitive, Vector};

pub fn register_xpbd_spatial_types(app: &mut App) {
    app.editor_registry::<RayCasterPrefab>();
    app.editor_into_sync::<RayCasterPrefab, RayCaster>();
    app.editor_registry::<ShapeCasterPrefab>();
    app.editor_into_sync::<ShapeCasterPrefab, ShapeCaster>();
}

#[derive(Component, Reflect, Clone, Debug, InspectorOptions)]
#[reflect(Component, Default)]
pub struct RayCasterPrefab {
    pub direction: Vector,
    pub origin: Vector,
}

impl Default for RayCasterPrefab {
    fn default() -> Self {
        Self {
            direction: Vector::X,
            origin: Vector::ZERO,
        }
    }
}

impl From<RayCasterPrefab> for RayCaster {
    fn from(val: RayCasterPrefab) -> Self {
        Self::new(val.origin, val.direction)
    }
}

#[derive(Component, Reflect, Clone, Debug, InspectorOptions, Default)]
#[reflect(Component, Default)]
pub struct ShapeCasterPrefab {
    pub shape: ColliderPrimitive,
    pub origin: Vector,
    pub direction: Vector,
    pub shape_rotation: Quaternion,
}

impl From<ShapeCasterPrefab> for ShapeCaster {
    fn from(val: ShapeCasterPrefab) -> Self {
        Self::new(
            val.shape.to_collider(),
            val.origin,
            val.shape_rotation,
            val.direction,
        )
        .with_ignore_origin_penetration(true)
    }
}
