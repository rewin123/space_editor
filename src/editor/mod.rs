//code only for editor gui

use bevy::prelude::*;

pub mod inspector;
pub mod bot_menu;
pub mod hierarchy;
pub mod selected;
pub mod asset_insector;
use bevy_egui::EguiContexts;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_mod_picking::{prelude::*, PickableBundle};
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin, PanOrbitCameraSystemSet};

use crate::{EditorState, EditorSet, prefab::save::SaveState, PrefabMarker};

use self::prelude::Selected;

pub mod prelude {
    pub use super::inspector::*;
    pub use super::bot_menu::*;
    pub use super::hierarchy::*;
    pub use super::selected::*;
    pub use bevy_panorbit_camera::{PanOrbitCamera};
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<bevy_egui::EguiPlugin>() {
            app.add_plugins(bevy_egui::EguiPlugin);
        }
        
        app
        .add_plugins(DefaultInspectorConfigPlugin)
        .add_plugins(prelude::SpaceHierarchyPlugin::default())
        .add_plugins(prelude::InspectorPlugin)
        .add_plugins(prelude::BotMenuPlugin)
        .add_plugins(PanOrbitCameraPlugin);

        if !app.is_plugin_added::<bevy_mod_picking::prelude::SelectionPlugin>() {
            app.add_plugins(bevy_mod_picking::DefaultPickingPlugins.build()
                .disable::<DebugPickingPlugin>()
                .disable::<DefaultHighlightingPlugin>());
        }

        app.insert_resource(PanOrbitEnabled(true));

        app.add_systems(Startup, (set_start_state, apply_state_transition::<EditorState>).chain().in_set(EditorSet::Editor));

        app.add_systems(Update, reset_pan_orbit_state
            .in_set(EditorSet::Editor));
        app.add_systems(Update, update_pan_orbit
            .after(reset_pan_orbit_state)
            .before(PanOrbitCameraSystemSet)
            .in_set(EditorSet::Editor));
        app.add_systems(Update, ui_camera_block.after(reset_pan_orbit_state).
            before(update_pan_orbit)
            .in_set(EditorSet::Editor));

        //play systems
        app.add_systems(OnEnter(EditorState::GamePrepare), save_prefab_before_play);
        app.add_systems(OnEnter(SaveState::Idle), to_game_after_save.run_if(in_state(EditorState::GamePrepare)));

        app.add_systems(OnEnter(EditorState::Editor), clear_and_load_on_start);

        app.add_systems(PostUpdate, 
            (auto_add_picking, 
                select_listener,
                auto_add_picking_dummy)
                .run_if(in_state(EditorState::Editor)));

        app.add_event::<SelectEvent>();
    }
}

#[derive(Event)]
struct SelectEvent {
    e : Entity,
    event : ListenerInput<Pointer<Down>>
}



fn auto_add_picking(
    mut commands : Commands,
    query : Query<Entity, (With<PrefabMarker>, Without<Pickable>)>
) {
    for e in query.iter() {
        commands.entity(e).insert(PickableBundle::default())
            .insert(RaycastPickTarget::default())
            .insert(On::<Pointer<Down>>::send_event::<SelectEvent>());
    }
}

fn auto_add_picking_dummy(
    mut commands : Commands,
    query : Query<Entity, (Without<PrefabMarker>, Without<Pickable>, With<Parent>)>
) {
    for e in query.iter() {
        commands.entity(e).insert(PickableBundle::default())
            .insert(RaycastPickTarget::default());
    }
}

fn select_listener(
    mut commands : Commands,
    query : Query<Entity, With<Selected>>,
    mut events : EventReader<SelectEvent>,
    mut ctxs : EguiContexts,
    keyboard: Res<Input<KeyCode>>,
) {
    if ctxs.ctx_mut().is_pointer_over_area() || ctxs.ctx_mut().is_using_pointer() {
        return;
    }
    for event in events.iter() {
        match event.event.button {
            PointerButton::Primary => {
                commands.entity(event.e).insert(Selected);
                if !keyboard.pressed(KeyCode::ShiftLeft) {
                    for e in query.iter() {
                        commands.entity(e).remove::<Selected>();
                    }
                }
            },
            PointerButton::Secondary => {/*Show context menu?*/},
            PointerButton::Middle => {},
        }
    }
}

impl From<ListenerInput<Pointer<Down>>> for SelectEvent {
    fn from(value: ListenerInput<Pointer<Down>>) -> Self {
        SelectEvent { e: value.listener(), event: value }
    }
}

fn save_prefab_before_play(
    mut save_state : ResMut<NextState<SaveState>>,
) {
    save_state.set(SaveState::Save);
}

fn to_game_after_save(
    mut state : ResMut<NextState<EditorState>>
) {
    state.set(EditorState::Game);
}

fn set_start_state(
    mut state : ResMut<NextState<EditorState>>
) {
    state.set(EditorState::Editor);
}

fn clear_and_load_on_start(
    mut load_server : ResMut<prelude::EditorLoader>,
    save_confg : Res<crate::prefab::save::SaveConfig>,
    assets : Res<AssetServer>,
) {
    if save_confg.path.is_empty() {
        return;
    }
    load_server.scene = Some(
        assets.load(format!("{}_recover.scn.ron",save_confg.path))
    );
}

#[derive(Resource, Default)]
pub struct PanOrbitEnabled(pub bool);


pub fn reset_pan_orbit_state(
    mut state : ResMut<PanOrbitEnabled>
) {
    *state = PanOrbitEnabled(true);
}

pub fn update_pan_orbit(
    mut pan_orbit_query: Query<&mut PanOrbitCamera>,
    state : Res<PanOrbitEnabled>,
) {
    for mut pan_orbit in pan_orbit_query.iter_mut() {
        pan_orbit.enabled = state.0;
    }    
}

pub fn ui_camera_block(
    mut ctxs : EguiContexts,
    mut state : ResMut<PanOrbitEnabled>
) {
    if ctxs.ctx_mut().is_pointer_over_area() || ctxs.ctx_mut().is_using_pointer() {
        *state = PanOrbitEnabled(false);
    }
}
