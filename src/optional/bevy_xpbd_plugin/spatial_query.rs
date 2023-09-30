use bevy::prelude::*;
use bevy_inspector_egui::*;
use bevy_xpbd_3d::prelude::*;

use crate::{prelude::EditorRegistryExt, EditorSet, EditorState};

use super::Vector;

pub fn register_xpbd_spatial_types(app : &mut App) {
    app.editor_registry::<RayCasterPrefab>();

    app.editor_into_sync::<RayCasterPrefab, RayCaster>();

    app.add_systems(Update, draw_ray_caster.in_set(EditorSet::Editor).run_if(in_state(EditorState::Editor)));
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
            origin: Vector::ZERO
         }
    }
}

impl Into<RayCaster> for RayCasterPrefab {
    fn into(self) -> RayCaster {
        RayCaster::new(self.origin, self.direction)
    }
}

fn ray_caster_prefab_change_fix(
    mut query : Query<&mut RayCasterPrefab, Changed<RayCasterPrefab>>
) {
    for mut prefab in query.iter_mut() {
        prefab.direction = prefab.direction.normalize();
    }
}

//debug in editor draw
fn draw_ray_caster(
    mut gizmos : Gizmos,
    query : Query<(&RayCaster, &RayHits)>
) {
    for (ray, hits) in query.iter() {
        let global_origin = ray.global_origin();
        let global_direction = ray.global_direction();
        for hit in hits.iter() {
            gizmos.line(
                global_origin,
                global_origin + global_direction * hit.time_of_impact,
                Color::PURPLE
            );
            gizmos.sphere(
                global_origin + global_direction * hit.time_of_impact,
                Quat::IDENTITY,
                0.1,
                Color::PURPLE
            );
        }

        if hits.is_empty() {
            let inf_color = Color::GRAY;
            gizmos.line(
                global_origin,
                global_origin + global_direction * 1000.0,
                inf_color
            );
        }
        
    }
}