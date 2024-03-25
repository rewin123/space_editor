use bevy::{prelude::*, utils::HashMap};
use bevy_egui::egui;

use crate::{EditorTab, EditorTabName};

/// Temporary resource for pretty system, based tab registration
pub struct EditorUiRef(pub egui::Ui);

pub struct ScheduleEditorTab {
    pub schedule: Schedule,
    pub title: egui::WidgetText,
}

impl EditorTab for ScheduleEditorTab {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut Commands, world: &mut World) {
        let inner_ui = ui.child_ui(ui.max_rect(), *ui.layout());
        world.insert_non_send_resource(EditorUiRef(inner_ui));

        self.schedule.run(world);
        world.remove_non_send_resource::<EditorUiRef>();
    }

    fn title(&self) -> egui::WidgetText {
        self.title.clone()
    }
}

#[derive(Resource, Default)]
pub struct ScheduleEditorTabStorage(pub HashMap<EditorTabName, ScheduleEditorTab>);
