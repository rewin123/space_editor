//code only for editor gui

use bevy::prelude::*;

pub mod inspector;
pub mod bot_menu;
pub mod hierarchy;
pub mod selected;
pub mod asset_insector;
use bevy_egui::EguiContexts;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin, PanOrbitCameraSystemSet};

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
        
        app.add_plugins(prelude::SpaceHierarchyPlugin::default())
        .add_plugins(prelude::InspectorPlugin)
        .add_plugins(prelude::BotMenuPlugin)
        .add_plugins(PanOrbitCameraPlugin);

        app.insert_resource(PanOrbitEnabled(true));

        app.add_systems(Update, reset_pan_orbit_state);
        app.add_systems(Update, update_pan_orbit.after(reset_pan_orbit_state).before(PanOrbitCameraSystemSet));
        app.add_systems(Update, ui_camera_block.after(reset_pan_orbit_state).before(update_pan_orbit));
    }
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