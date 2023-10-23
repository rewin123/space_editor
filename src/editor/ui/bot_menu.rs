use bevy::prelude::*;
use bevy_egui::*;

use crate::{
    prefab::PrefabPlugin,
    prelude::{EditorEvent, EditorPrefabPath},
    EditorSet, EditorState,
};

#[derive(Resource, Default, Clone)]
pub struct EditorLoader {
    pub scene: Option<Handle<DynamicScene>>,
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

        app.add_systems(Update, bot_menu.in_set(EditorSet::Editor));
        app.add_systems(Update, bot_menu_game.in_set(EditorSet::Game));
        app.add_event::<MenuLoadEvent>();
    }
}

#[derive(Event)]
pub struct MenuLoadEvent {
    pub path: String,
}

fn bot_menu_game(
    mut smoothed_dt: Local<f32>,
    mut ctxs: EguiContexts,
    mut state: ResMut<NextState<EditorState>>,
    time: Res<Time>,
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
    pub file_dialog: Option<egui_file::FileDialog>,
    pub gltf_dialog: Option<egui_file::FileDialog>,
    pub path: String,
}

pub fn bot_menu(
    mut ctxs: EguiContexts,
    _state: ResMut<NextState<EditorState>>,
    mut events: EventReader<MenuLoadEvent>,
    mut menu_state: ResMut<BotMenuState>,
    mut editor_events: EventWriter<EditorEvent>,
) {
    let ctx = ctxs.ctx_mut();
    egui::TopBottomPanel::bottom("bot menu").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label("Save path:");
            ui.add(egui::TextEdit::singleline(&mut menu_state.path));

            if ui.button("📂").clicked() {
                let mut dialog = egui_file::FileDialog::open_file(Some("assets/".into()))
                    .filter(Box::new(|path| {
                        path.to_str().unwrap().ends_with(".scn.ron")
                    }))
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
                            menu_state.path = path;
                            editor_events.send(EditorEvent::Load(EditorPrefabPath::File(format!(
                                "{}.scn.ron",
                                menu_state.path.clone()
                            ))));
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

            if let Some(gltf_dialog) = &mut menu_state.gltf_dialog {
                if gltf_dialog.show(ctx).selected() {
                    if let Some(file) = gltf_dialog.path() {
                        let mut path = file.to_str().unwrap().to_string();
                        //remove assets/ from path
                        if path.starts_with("assets/") {
                            path = path.replace("assets/", "");

                            editor_events.send(EditorEvent::LoadGltfAsPrefab(path.to_string()));
                        }
                    }
                } else {
                    let mut need_move_to_default_dir = false;
                    if let Some(path) = gltf_dialog.path() {
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
                        gltf_dialog.set_path("assets/");
                    }
                }
            }

            if ui.button("Save").clicked() {
                editor_events.send(EditorEvent::Save(EditorPrefabPath::File(format!(
                    "{}.scn.ron",
                    menu_state.path.clone()
                ))));
            }

            if ui.button("Load").clicked() && !menu_state.path.is_empty() {
                editor_events.send(EditorEvent::Load(EditorPrefabPath::File(format!(
                    "{}.scn.ron",
                    menu_state.path.clone()
                ))));
                // load_server.scene = Some(
                //     assets.load(format!("{}.scn.ron",save_confg.path))
                // );
            }

            if ui.button("Open gltf as prefab").clicked() {
                let mut gltf_dialog = egui_file::FileDialog::open_file(Some("assets/".into()))
                    .filter(Box::new(|path| {
                        path.to_str().unwrap().ends_with(".gltf")
                            || path.to_str().unwrap().ends_with(".glb")
                    }))
                    .title("Open gltf scene");
                gltf_dialog.open();
                menu_state.gltf_dialog = Some(gltf_dialog);
                // editor_events.send(EditorEvent::LoadGltfAsPrefab(
                //     "low_poly_fighter_2.gltf".to_string()
                // ));
            }

            if ui.button("▶").clicked() {
                editor_events.send(EditorEvent::StartGame);
            }
        });
    });

    for event in events.iter() {
        menu_state.path = event.path.clone();
        editor_events.send(EditorEvent::Load(EditorPrefabPath::File(format!(
            "{}.scn.ron",
            menu_state.path.clone()
        ))));
    }
    events.clear();
}
