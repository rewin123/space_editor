use bevy::prelude::*;

use bevy_egui::{egui::RichText, *};
use bevy_inspector_egui::bevy_egui;
use space_editor_ui::editor_tab::EditorTab;

use crate::{heightmap::MapSettings, UpdateTerrain};

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
    ui.heading("Terrain Map Generator");

    let mut query = world.query::<(Entity, Option<&Name>, &mut MapSettings)>();

    let type_registry = world.resource::<AppTypeRegistry>().clone();

    let mut events = vec![];

    for (entity, name, mut map) in query.iter_mut(world) {
        if let Some(name) = name {
            ui.label(format!("{}: ", name.as_str()));
        } else {
            ui.label(format!("Terrain {:?}: ", entity));
        }

        ui.push_id(format!("terrain-{:?}", &entity), |ui| {
            bevy_inspector_egui::reflect_inspector::ui_for_value(
                map.as_mut(),
                ui,
                &type_registry.read(),
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
            events.push(UpdateTerrain::One(entity));
        }
        if reset_button.clicked() {
            *map = MapSettings::default();
            events.push(UpdateTerrain::One(entity));
        }
        ui.separator();
    }

    for event in events {
        world.send_event(event);
    }
}
