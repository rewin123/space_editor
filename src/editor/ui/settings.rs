use bevy::prelude::*;
use bevy_egui::*;

use crate::prelude::EditorTab;

pub struct SettingsWindow {}

impl EditorTab for SettingsWindow {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World) {
        ui.label("Settings");
    }

    fn title(&self) -> egui::WidgetText {
        "Settings".into()
    }
}
