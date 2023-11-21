use bevy::prelude::*;
use bevy_inspector_egui::*;
use bevy_xpbd_3d::{math::Quaternion, prelude::*};

use crate::{prelude::EditorRegistryExt, EditorSet, EditorState};

use super::{collider::ColliderPrimitive, Vector};

pub fn register_xpbd_spatial_types(app: &mut App) {
    app.editor_registry::<RayCasterPrefab>();
    app.editor_into_sync::<RayCasterPrefab, RayCaster>();
    app.add_systems(
        Update,
        draw_ray_caster
            .in_set(EditorSet::Editor)
            .run_if(in_state(EditorState::Editor)),
    );

    app.editor_registry::<ShapeCasterPrefab>();
    app.editor_into_sync::<ShapeCasterPrefab, ShapeCaster>();
    app.add_systems(
        Update,
        draw_shapecast
            .in_set(EditorSet::Editor)
            .run_if(in_state(EditorState::Editor)),
    );
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

//debug in editor draw
fn draw_ray_caster(mut gizmos: Gizmos, query: Query<(&RayCaster, &RayHits)>) {
    for (ray, hits) in query.iter() {
        let global_origin = ray.global_origin();
        let global_direction = ray.global_direction();
        for hit in hits.iter() {
            gizmos.line(
                global_origin,
                global_origin + global_direction * hit.time_of_impact,
                Color::PURPLE,
            );
            gizmos.sphere(
                global_origin + global_direction * hit.time_of_impact,
                Quat::IDENTITY,
                0.1,
                Color::PURPLE,
            );
        }

        if hits.is_empty() {
            let inf_color = Color::GRAY;
            gizmos.line(
                global_origin,
                global_origin + global_direction * 1000.0,
                inf_color,
            );
        }
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

fn draw_shapecast(mut gizmos: Gizmos, query: Query<(&ShapeCaster, &ShapeHits)>) {
    for (caster, hits) in query.iter() {
        let global_origin = caster.global_origin();
        let global_direction = caster.global_direction();
        for hit in hits.iter() {
            gizmos.line(
                global_origin,
                global_origin + global_direction * hit.time_of_impact,
                Color::PURPLE,
            );
            gizmos.sphere(
                global_origin + global_direction * hit.time_of_impact,
                Quat::IDENTITY,
                0.1,
                Color::PURPLE,
            );
        }

        if hits.is_empty() {
            let inf_color = Color::GRAY;
            gizmos.line(
                global_origin,
                global_origin + global_direction * 1000.0,
                inf_color,
            );
        }
    }
}
