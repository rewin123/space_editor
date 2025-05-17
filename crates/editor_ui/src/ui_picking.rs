use bevy::{prelude::*, window::PrimaryWindow};

use bevy_egui::*;
use space_shared::EditorSet;

use crate::prelude::EditorCameraEnabled;


pub struct UiPickingPlugin;

impl Plugin for UiPickingPlugin {
    fn build(&self, app: &mut App) {

        app.configure_sets(
            Update,
                UpdateNonUIAreas.in_set(EditorSet::Editor)
        );

        app.init_resource::<NonUIAreas>();

        app.add_systems(Update, clear_non_ui_areas.before(UpdateNonUIAreas).in_set(EditorSet::Editor));
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub struct UpdateNonUIAreas;


#[derive(Resource, Default)]
pub struct NonUIAreas {
    pub areas: Vec<egui::Rect>,
}

pub fn clear_non_ui_areas(mut non_ui_areas: ResMut<NonUIAreas>) {
    non_ui_areas.areas.clear();
}




