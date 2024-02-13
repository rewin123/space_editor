use bevy::{prelude::*, utils::HashMap};

use bevy_egui_next::*;
use space_shared::ext::bevy_inspector_egui;

use crate::prelude::*;

#[derive(Resource, Default)]
pub struct ResourceTab {
    open_resources: HashMap<String, bool>,
}

impl EditorTab for ResourceTab {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut Commands, world: &mut World) {
        inspect(ui, world, &mut self.open_resources);
    }

    fn title(&self) -> egui::WidgetText {
        "Resource".into()
    }
}

pub fn inspect(ui: &mut egui::Ui, world: &mut World, open_resources: &mut HashMap<String, bool>) {
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = type_registry.read();

    let mut resources: Vec<_> = type_registry
        .iter()
        .filter(|registration| registration.data::<ReflectResource>().is_some())
        .map(|registration| {
            (
                registration
                    .type_info()
                    .type_path_table()
                    .short_path()
                    .to_string(),
                registration.type_id(),
            )
        })
        .collect();
    resources.sort_by(|(name_a, _), (name_b, _)| name_a.cmp(name_b));

    egui::Grid::new("Resources ID".to_string()).show(ui, |ui| {
        for (resource_name, type_id) in resources {
            ui.push_id(format!("{:?}-{}", &type_id, &resource_name), |ui| {
                let header = egui::CollapsingHeader::new(resource_name.clone())
                    .default_open(*open_resources.get(&resource_name).unwrap_or(&false))
                    .show(ui, |ui| {
                        ui.push_id(format!("content-{:?}-{}", &type_id, &resource_name), |ui| {
                            bevy_inspector_egui::bevy_inspector::by_type_id::ui_for_resource(
                                world,
                                type_id,
                                ui,
                                &resource_name,
                                &type_registry,
                            );
                        });
                    });
                if header.header_response.clicked() {
                    let open_name = open_resources.entry(resource_name.clone()).or_default();
                    //At click header not opened simultaneously so its need to check percent of opened
                    *open_name = header.openness < 0.5;
                }
            });
            ui.end_row();
        }
    });
}
