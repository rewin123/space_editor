

use bevy::{prelude::*};
use bevy_egui::*;

use crate::{prefab::{save::{SaveState, SaveConfig}, PrefabPlugin}, prelude::{EditorEvent}, EditorState, EditorSet};

#[derive(Resource, Default, Clone)]
pub struct EditorLoader {
    pub scene : Option<Handle<DynamicScene>>
}

/// Plugin to activate bot menu in editor UI
pub struct BotMenuPlugin;

impl Plugin for BotMenuPlugin {
    fn build(&self, app: &mut App) {

        if !app.is_plugin_added::<PrefabPlugin>() {
            app.add_plugins(PrefabPlugin);
        }
        app.init_resource::<EditorLoader>();
        app.init_resource::<BotMenuState>();

        app.add_systems(Update, bot_menu
            .in_set(EditorSet::Editor));
        app.add_systems(Update, bot_menu_game.in_set(EditorSet::Game));
        app.add_event::<LoadEvent>();
    }
}

#[derive(Event)]
pub struct LoadEvent {
    pub path : String
}

fn bot_menu_game(
    mut smoothed_dt : Local<f32>,
    mut ctxs : EguiContexts,
    mut state : ResMut<NextState<EditorState>>,
    time : Res<Time>
) {
    egui::TopBottomPanel::bottom("bot_panel").show(ctxs.ctx_mut(), |ui| {
        ui.vertical_centered(|ui| {
            if ui.button("⏸").clicked() {
                state.set(EditorState::Editor);
            }

            *smoothed_dt = *smoothed_dt * 0.98 + time.delta_seconds() * 0.02;
            ui.label(format!("FPS: {:.0}", 1.0 / *smoothed_dt));
        });
    });
}

#[derive(Resource, Default)]
pub struct BotMenuState {
    pub file_dialog : Option<egui_file::FileDialog>
}

pub fn bot_menu(
    mut ctxs : EguiContexts,
    mut save_confg : ResMut<SaveConfig>,
    _save_state : ResMut<NextState<SaveState>>,
    assets : Res<AssetServer>,
    mut load_server : ResMut<EditorLoader>,
    mut state : ResMut<NextState<EditorState>>,
    mut events : EventReader<LoadEvent>,
    mut menu_state : ResMut<BotMenuState>,
    mut editor_events : EventWriter<EditorEvent>
) {
    let ctx = ctxs.ctx_mut();
    egui::TopBottomPanel::bottom("bot menu").show(ctx, |ui| {

        ui.horizontal(|ui| {

            ui.label("Save path:");
            ui.add(egui::TextEdit::singleline(&mut save_confg.path));

            if ui.button("📂").clicked() {
                let mut dialog = egui_file::FileDialog::open_file(Some("assets/".into()))
                    .filter(Box::new(|path| path.to_str().unwrap().ends_with(".scn.ron")))
                    .title("Open prefab (*.scn.ron)");
                dialog.open();
                menu_state.file_dialog = Some(dialog);
            }

            if let Some(dialog) = &mut menu_state.file_dialog {
                if dialog.show(ctx).selected() {
                    if let Some(file) = dialog.path() {
                        let mut path = file.to_str().unwrap().to_string();
                        //remove assets/ from path
                        if path.starts_with("assets/") {
                            path = path.replace("assets/", "");
                            //remove .scn.ron
                            path = path.replace(".scn.ron", "");
                            save_confg.path = path;
                            load_server.scene = Some(
                                assets.load(format!("{}.scn.ron",save_confg.path))
                            );
                        }
                    }
                } else {
                    let mut need_move_to_default_dir = false;
                    if let Some(path) = dialog.path() {
                        if let Some(path) = path.to_str() {
                            if !path.contains("assets") {
                                need_move_to_default_dir = true;
                            }
                        } else {
                            need_move_to_default_dir = true;
                        }
                    } else {
                        need_move_to_default_dir = true;
                    }
                    if need_move_to_default_dir {
                        dialog.set_path("assets/");
                    }
                }
            }

            if ui.button("Save").clicked() {
                
                editor_events.send(EditorEvent::Save(save_confg.path.clone()));
            }

            if ui.button("Load").clicked() && !save_confg.path.is_empty() {
                editor_events.send(EditorEvent::Load(format!("{}.scn.ron",save_confg.path)));
                // load_server.scene = Some(
                //     assets.load(format!("{}.scn.ron",save_confg.path))
                // );
            }

            if ui.button("▶").clicked() {
                state.set(EditorState::GamePrepare);
            }
        });
    });

    for event in events.iter() {
        save_confg.path = event.path.clone();
        load_server.scene = Some(
            assets.load(format!("{}.scn.ron",save_confg.path))
        );
    } 
    events.clear();
}
