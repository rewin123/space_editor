use crate::*;
use bevy::prelude::*;

pub struct EditorPickingPlugin;

impl Plugin for EditorPickingPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_mod_picking::DefaultPickingPlugins);

        app.world
            .resource_mut::<bevy_mod_picking::backends::raycast::RaycastBackendSettings>()
            .require_markers = true;

        app.add_systems(
            Update,
            (auto_add_picking, select_listener).after(UiSystemSet::Last),
        );
        app.add_systems(PostUpdate, auto_add_picking_dummy);
    }
}

pub fn auto_add_picking(
    mut commands: Commands,
    query: Query<Entity, (With<PrefabMarker>, Without<Pickable>)>,
) {
    for e in query.iter() {
        commands
            .entity(e)
            .insert(PickableBundle::default())
            .insert(On::<Pointer<Down>>::send_event::<SelectEvent>())
            .insert(RaycastPickable);
    }
}

//Auto add picking for each child to propagate picking event up to prefab entitiy
pub fn auto_add_picking_dummy(
    mut commands: Commands,
    query: Query<(Entity, &Handle<Mesh>), AutoAddQueryFilter>,
    meshs: Res<Assets<Mesh>>,
) {
    for (e, mesh) in query.iter() {
        //Only meshed entity need to be pickable
        if let Some(mesh) = meshs.get(mesh) {
            if mesh.primitive_topology() == PrimitiveTopology::TriangleList {
                commands
                    .entity(e)
                    .insert(PickableBundle::default())
                    .insert(RaycastPickable);
            }
        }
    }
}

pub fn select_listener(
    mut commands: Commands,
    query: Query<Entity, With<Selected>>,
    // may need to be optimized a bit so that there is less overlap
    query_parent: Query<&SelectParent>,
    mut events: EventReader<SelectEvent>,
    pan_orbit_state: ResMut<EditorCameraEnabled>,
    keyboard: Res<Input<KeyCode>>,
) {
    if !pan_orbit_state.0 {
        events.clear();
        return;
    }
    for event in events.read() {
        info!("Select Event: {:?}", event.e);
        let entity = query_parent.get(event.e).map_or(event.e, |a| a.parent);
        match event.event.button {
            PointerButton::Primary => {
                commands.entity(entity).insert(Selected);
                if !keyboard.pressed(KeyCode::ShiftLeft) {
                    for e in query.iter() {
                        commands.entity(e).remove::<Selected>();
                    }
                }
            }
            PointerButton::Secondary => { /*Show context menu?*/ }
            PointerButton::Middle => {}
        }
    }
}

pub fn delete_selected(
    mut commands: Commands,
    query: Query<Entity, With<Selected>>,
    keyboard: Res<Input<KeyCode>>,
) {
    let shift = keyboard.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let ctrl = keyboard.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    let delete = keyboard.just_pressed(KeyCode::Back) || keyboard.just_pressed(KeyCode::Delete);

    if ctrl && shift && delete {
        for entity in query.iter() {
            info!("Delete Entity: {entity:?}");
            commands.entity(entity).despawn_recursive();
        }
    }
}

impl From<ListenerInput<Pointer<Down>>> for SelectEvent {
    fn from(value: ListenerInput<Pointer<Down>>) -> Self {
        Self {
            e: value.target(),
            event: value,
        }
    }
}

/// This event used for selecting entities
#[derive(Event, Clone, EntityEvent)]
pub struct SelectEvent {
    #[target]
    e: Entity,
    event: ListenerInput<Pointer<Down>>,
}
