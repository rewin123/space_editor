use bevy::prelude::*;
use bevy_egui::egui;
use space_editor_tabs::prelude::*;

use crate::editor_tab_name::EditorTabName;

#[derive(Resource)]
pub struct DebugWorldInspector {}

impl EditorTab for DebugWorldInspector {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut Commands, world: &mut World) {
        bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);
    }

    fn tab_name(&self) -> space_editor_tabs::tab_name::TabNameHolder {
        EditorTabName::DebugWorldInspector.into()
    }
}
