use super::editor_tab::*;
use bevy::prelude::*;
use bevy_egui::egui;

#[derive(Resource)]
pub struct DebugWorldInspector {}

impl EditorTab for DebugWorldInspector {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut Commands, world: &mut World) {
        bevy_inspector_egui::bevy_inspector::ui_for_world(world, ui);
    }

    fn title(&self) -> egui::WidgetText {
        "Debug World Inspector".into()
    }
}
