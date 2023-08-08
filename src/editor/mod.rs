//code only for editor gui

use bevy::prelude::*;

pub mod inspector;
pub mod bot_menu;
pub mod hierarchy;
pub mod selected;
pub mod asset_insector;
use bevy_egui::EguiContexts;
use bevy_inspector_egui::DefaultInspectorConfigPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin, PanOrbitCameraSystemSet};

use crate::{EditorState, EditorSet, prefab::save::SaveState};

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