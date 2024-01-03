use bevy::prelude::*;

use bevy_egui::*;

use crate::prelude::*;

#[derive(Resource, Default)]
pub struct EventTab;

impl EditorTab for EventTab {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut Commands, world: &mut World) {
        inspect(ui, world);
    }

    fn title(&self) -> egui::WidgetText {
        "Event".into()
    }
}

pub fn inspect(ui: &mut egui::Ui, world: &mut World) {
    let mut events = world
        .resource::<EditorRegistry>()
        .send_events
        .iter()
        .map(|(type_id, event)| (*type_id, event.clone()))
        .collect::<Vec<_>>();

    events.sort_by(|(_, a), (_, b)| a.name().cmp(b.name()));

    egui::Grid::new("Events ID".to_string()).show(ui, move |ui| {
        for (type_id, event) in events.into_iter() {
            ui.push_id(format!("{:?}-{}", &type_id, event.path()), |ui| {
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
