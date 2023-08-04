use bevy::prelude::*;
use bevy_egui::*;

use crate::{prefab::{save_load::{SaveState, SaveConfig}, PrefabPlugin}, inspector};


pub struct TopMenuPlugin;

impl Plugin for TopMenuPlugin {
    fn build(&self, app: &mut App) {

        if !app.is_plugin_added::<PrefabPlugin>() {
            app.add_plugins(PrefabPlugin);
        }

        app.add_systems(Update, top_menu.after(inspector::inspect));
    }
}

fn top_menu(
    mut commands : Commands,
    mut ctxs : EguiContexts,
    mut save_confg : ResMut<SaveConfig>,
    mut save_state : ResMut<NextState<SaveState>>
) {
    let ctx = ctxs.ctx_mut();
    egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {

            ui.label("Save path:");
            ui.add(egui::TextEdit::singleline(&mut save_confg.path));

            if ui.button("Save").clicked() {
                save_state.set(SaveState::Save);
            }

            if ui.button("Load").clicked() {
                
            }
        });
    });
}