use bevy::prelude::*;

use bevy_egui::{egui::RichText, *};
use terraingen::TerrainMap;

use crate::prelude::EditorTab;

#[derive(Resource, Default)]
pub struct TerrainGenView;

impl EditorTab for TerrainGenView {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut Commands, world: &mut World) {
        inspect(ui, world);
    }

    fn title(&self) -> egui::WidgetText {
        "Terrain Generator".into()
    }
}

pub fn inspect(ui: &mut egui::Ui, world: &mut World) {
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = type_registry.read();

    let resource = type_registry
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
        .find(|res| res.0 == "TerrainMap");

    if let Some((resource_name, type_id)) = resource {
        ui.heading("Terrain Map Generator");
        ui.separator();
        ui.push_id(format!("content-{:?}-{}", &type_id, &resource_name), |ui| {
            bevy_inspector_egui::bevy_inspector::by_type_id::ui_for_resource(
                world,
                type_id,
                ui,
                &resource_name,
                &type_registry,
            );
        });

        ui.add_space(8.0);
        // TODO: Add button for terrain data/mesh persistance
        let redraw_button = ui.button(
            RichText::new("Redraw Terrain")
                .line_height(Some(20.))
                .size(16.),
        );
        let reset_button = ui.button(
            RichText::new("Reset Terrain")
                .line_height(Some(20.))
                .size(16.),
        );

        if redraw_button.clicked() {
            let mut map = world.resource_mut::<TerrainMap>();
            map.has_changes = true;
        }
        if reset_button.clicked() {
            let mut map = world.resource_mut::<TerrainMap>();
            *map = TerrainMap::default();
            map.has_changes = true;
        }
    }
}
