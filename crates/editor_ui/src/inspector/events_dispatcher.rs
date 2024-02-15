use bevy::{prelude::*, utils::HashMap};

use bevy_egui_next::*;
use space_shared::ext::bevy_inspector_egui;

use crate::{colors::ERROR_COLOR, prelude::*};

#[derive(Resource, Default)]
pub struct EventDispatcherTab {
    open_events: HashMap<String, bool>,
}

impl EditorTab for EventDispatcherTab {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut Commands, world: &mut World) {
        inspect(ui, world, &mut self.open_events);
    }

    fn title(&self) -> egui::WidgetText {
        "Event Dispatcher".into()
    }
}

pub fn inspect(ui: &mut egui::Ui, world: &mut World, open_events: &mut HashMap<String, bool>) {
    let events = &mut world.resource::<EditorRegistry>().send_events.clone();
    events.sort_by(|a, b| a.name().cmp(b.name()));

    if events.is_empty() {
        ui.label(egui::RichText::new("No events registered").color(ERROR_COLOR));
    } else {
        let type_registry = world.resource::<AppTypeRegistry>().clone();
        let type_registry = type_registry.read();

        egui::Grid::new("Events ID".to_string()).show(ui, move |ui| {
            for event in events {
                let header = egui::CollapsingHeader::new(event.name())
                    .default_open(*open_events.get(event.name()).unwrap_or(&false))
                    .show(ui, |ui| {
                        ui.push_id(
                            format!("event-{:?}-{}", &event.type_id, &event.name()),
                            |ui| {
                                bevy_inspector_egui::bevy_inspector::by_type_id::ui_for_resource(
                                    world,
                                    event.type_id,
                                    ui,
                                    event.name(),
                                    &type_registry,
                                );

                                let clicked = ui
                                    .button(format!("Send {}", event.name()))
                                    .on_hover_text(event.path())
                                    .clicked();
                                if clicked {
                                    event.send(world);
                                };
                            },
                        );
                    });
                if header.header_response.clicked() {
                    let open_name = open_events.entry(event.name().to_string()).or_default();
                    //At click header not opened simultaneously so its need to check percent of opened
                    *open_name = header.openness < 0.5;
                }

                ui.end_row();
            }
        });
    }
}
