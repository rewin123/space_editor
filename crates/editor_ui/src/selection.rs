use crate::*;
use bevy::{color::palettes::tailwind::{PINK_100, RED_500}, picking::pointer::PointerInteraction, prelude::*};



#[cfg(not(tarpaulin_include))]
pub fn plugin(app: &mut App) {
    app.add_plugins(MeshPickingPlugin);


    app.add_systems(
        Update,
        (delete_selected, reemit_pointer_click, auto_add_markers)
    );

    app.add_systems(
        Update,
        draw_mesh_intersections.run_if(in_state(EditorState::Editor))
    );

    app.add_event::<AddMarkersEvent>();

    app.add_observer(select_listener);
    app.add_observer(recursive_add_markers);

    app.insert_resource(MeshPickingSettings {
        require_markers: true,
        ray_cast_visibility: RayCastVisibility::VisibleInView
    });
}

fn auto_add_markers(
    mut commands: Commands,
    q_prefabs: Query<Entity, (With<PrefabMarker>, Without<RayCastPickable>)>,
    q_cameras: Query<Entity, (With<Camera3d>, Without<RayCastPickable>)>,
) {
    for entity in q_prefabs.iter() {
        commands.trigger_targets(AddMarkersEvent, entity);
    }

    for entity in q_cameras.iter() {
        commands.entity(entity).insert(RayCastPickable);
    }
}

#[derive(Event, Clone)]
struct AddMarkersEvent;

fn recursive_add_markers(
    trigger: Trigger<AddMarkersEvent>,
    q_children: Query<&Children>,
    q_meshes: Query<Entity, With<Mesh3d>>,
    mut commands: Commands,
) {
    if q_meshes.contains(trigger.entity()) {
        commands.entity(trigger.entity()).insert(RayCastPickable);
    }

    if let Ok(children) = q_children.get(trigger.entity()) {
        for child in children.iter() {
            commands.trigger_targets(AddMarkersEvent, *child);
        }
    }
}

/// From bevy examples
/// A system that draws hit indicators for every pointer.
fn draw_mesh_intersections(pointers: Query<&PointerInteraction>, mut gizmos: Gizmos) {
    for (point, normal) in pointers
        .iter()
        .filter_map(|interaction| interaction.get_nearest_hit())
        .filter_map(|(_entity, hit)| hit.position.zip(hit.normal))
    {
        gizmos.sphere(point, 0.05, RED_500);
        gizmos.arrow(point, point + normal.normalize() * 0.5, PINK_100);
    }
}

/// Reemits the pointer click event to the entity that is being clicked on
/// Its not a good solution, but it works for now
fn reemit_pointer_click(
    pointers: Query<&PointerInteraction>,
    mut commands: Commands,
    q_meshes: Query<Entity, With<Mesh3d>>,
) {
    for pointer in pointers.iter() {
        if let Some((e, _)) = pointer.get_nearest_hit() {
            if q_meshes.contains(*e) {
                commands.trigger_targets(SelectEvent, *e);
            }
        }
    }
}

pub fn select_listener(
    mut trigger: Trigger<SelectEvent>,
    mut commands: Commands,
    query: Query<Entity, With<Selected>>,
    // may need to be optimized a bit so that there is less overlap
    prefabs: Query<Entity, With<PrefabMarker>>,
    parents: Query<&Parent>,
    pan_orbit_state: ResMut<EditorCameraEnabled>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {

    if !pan_orbit_state.0 {
        trigger.propagate(false);
        return;
    }

    info!("Select Event: {:?}", trigger.entity());

    if let Ok(entity) = prefabs.get(trigger.entity()) {
        commands.entity(entity).insert(Selected);
        if !keyboard.pressed(KeyCode::ShiftLeft) {
            for e in query.iter() {
                commands.entity(e).remove::<Selected>();
            }
        }
    } else if let Ok(parent) = parents.get(trigger.entity()) {
        // Just stupid propagation (Need to make it with Event trait)
        commands.trigger_targets(SelectEvent, parent.get()); 
    }
}


/// This event used for selecting entities
#[derive(Event, Clone)]
pub struct SelectEvent;

pub fn delete_selected(
    mut commands: Commands,
    query: Query<Entity, With<Selected>>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    let shift = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let ctrl = keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    let delete = keyboard.any_just_pressed([KeyCode::Backspace, KeyCode::Delete]);

    if ctrl && shift && delete {
        for entity in query.iter() {
            info!("Delete Entity: {entity:?}");
            commands.entity(entity).despawn_recursive();
        }
    }
}

