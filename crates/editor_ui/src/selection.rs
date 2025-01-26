use crate::*;
use bevy::prelude::*;



#[cfg(not(tarpaulin_include))]
pub fn plugin(app: &mut App) {
    app.add_plugins(MeshPickingPlugin);

    app.add_systems(
        Update,
        delete_selected
    );

    app.add_observer(select_listener);
    app.add_observer(reemit_pointer_click);
}

fn reemit_pointer_click(
    mut trigger: Trigger<Pointer<Down>>,
    mut commands: Commands,
) {
    commands.trigger_targets(SelectEvent, trigger.entity());
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

