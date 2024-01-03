use bevy::prelude::*;

use bevy_egui::*;

use crate::prelude::*;

#[derive(Resource, Default)]
pub struct EventDebuggerTab;

impl EditorTab for EventDebuggerTab {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut Commands, world: &mut World) {
        inspect(ui, world);
    }

    fn title(&self) -> egui::WidgetText {
        "Event Debugger".into()
    }
}

pub fn inspect(ui: &mut egui::Ui, world: &mut World) {
    let events = &world.resource::<EditorRegistry>().send_events.clone();

    egui::Grid::new("Events ID".to_string()).show(ui, move |ui| {
        for event in events {
            ui.push_id(event.path(), |ui| {
                let clicked = ui
                    .button(event.name())
                    .on_hover_text(event.path())
                    .clicked();
                if clicked {
                    event.send(world);
                };
            });
            ui.end_row();
        }
    });
}
