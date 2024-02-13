use bevy::{asset::ReflectAsset, prelude::*, utils::HashMap};

use bevy_egui_next::*;
use space_shared::ext::bevy_inspector_egui;

use crate::prelude::*;

#[derive(Resource, Default)]
pub struct RuntimeAssetsTab {
    open_assets: HashMap<String, bool>,
}

impl EditorTab for RuntimeAssetsTab {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut Commands, world: &mut World) {
        inspect(ui, world, &mut self.open_assets);
    }

    fn title(&self) -> egui::WidgetText {
        "Runtime Assets".into()
    }
}

pub fn inspect(ui: &mut egui::Ui, world: &mut World, open_assets: &mut HashMap<String, bool>) {
    let type_registry = world.resource::<AppTypeRegistry>().clone();
    let type_registry = type_registry.read();

    let mut assets: Vec<_> = type_registry
        .iter()
        .filter_map(|registration| {
            let reflect_asset = registration.data::<ReflectAsset>()?;
            Some((
                registration
                    .type_info()
                    .type_path_table()
                    .short_path()
                    .to_string(),
                registration.type_id(),
                reflect_asset
                    .ids(world)
                    .find(|id| id.type_id() == registration.type_id())?,
            ))
        })
        .collect();
    assets.sort_by(|(name_a, _, _), (name_b, _, _)| name_a.cmp(name_b));

    egui::Grid::new("Assets ID".to_string()).show(ui, |ui| {
        for (asset_name, type_id, handle) in assets {
            ui.push_id(format!("{:?}-{}", &type_id, &asset_name), |ui| {
                let header = egui::CollapsingHeader::new(asset_name.clone())
                    .default_open(*open_assets.get(&asset_name).unwrap_or(&false))
                    .show(ui, |ui| {
                        ui.push_id(format!("content-{:?}-{}", &type_id, &asset_name), |ui| {
                            bevy_inspector_egui::bevy_inspector::by_type_id::ui_for_asset(
                                world,
                                type_id,
                                handle,
                                ui,
                                &type_registry,
                            );
                        });
                    });
                if header.header_response.clicked() {
                    let open_name = open_assets.entry(asset_name.clone()).or_default();
                    //At click header not opened simultaneously so its need to check percent of opened
                    *open_name = header.openness < 0.5;
                }
            });
            ui.end_row();
        }
    });
}
