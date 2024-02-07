use std::sync::Arc;

use bevy::prelude::*;
use bevy_egui::{
    egui::{Align, Color32, Stroke},
    *,
};
use space_editor_core::prelude::*;
use space_prefab::plugins::PrefabPlugin;
use space_shared::{ext::egui_file, *};
use space_undo::{AddedEntity, NewChange, RemovedEntity};

use crate::{
    hierarchy::{HierarchyQueryIter, HierarchyTabState},
    icons::{add_bundle_icon, add_entity_icon, delete_entity_icon},
    ui_registration::{BundleReg, EditorBundleUntyped},
};

/// Plugin to activate bottom menu in editor UI
pub struct BottomMenuPlugin;

impl Plugin for BottomMenuPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<PrefabPlugin>() {
            app.add_plugins(PrefabPlugin);
        }
        app.init_resource::<EditorLoader>();
        app.init_resource::<BottomMenuState>();

        app.add_systems(
            Update,
            bottom_menu.before(EditorLoadSet).in_set(EditorSet::Editor),
        );
        app.add_systems(
            Update,
            top_menu.before(EditorLoadSet).in_set(EditorSet::Editor),
        );
        app.add_systems(Update, in_game_menu.in_set(EditorSet::Game));
        app.add_event::<MenuLoadEvent>();
    }
}

#[derive(Event)]
pub struct MenuLoadEvent {
    pub path: String,
}

fn in_game_menu(
    mut smoothed_dt: Local<f32>,
    mut ctxs: EguiContexts,
    mut state: ResMut<NextState<EditorState>>,
    mut time: ResMut<Time<Virtual>>,
) {
    egui::TopBottomPanel::top("top_gameplay_panel")
        .exact_height(28.)
        .show(ctxs.ctx_mut(), |ui| {
            *smoothed_dt = (*smoothed_dt).mul_add(0.98, time.delta_seconds() * 0.02);
            let layout = egui::Layout::left_to_right(Align::Center).with_main_align(Align::Center);
            ui.with_layout(layout, |ui| {
                if ui.button("⏸").clicked() {
                    if time.is_paused() {
                        time.unpause();
                    } else {
                        time.pause();
                    }
                }
                if ui.button("⏹").clicked() {
                    state.set(EditorState::Editor);
                }
                ui.spacing();
                ui.label(format!("FPS: {:.0}", 1.0 / *smoothed_dt));
            });
        });
}

#[derive(Resource, Default)]
pub struct BottomMenuState {
    pub file_dialog: Option<egui_file::FileDialog>,
    pub gltf_dialog: Option<egui_file::FileDialog>,
    pub path: String,
}

pub fn bottom_menu(
    mut commands: Commands,
    query: Query<HierarchyQueryIter, With<PrefabMarker>>,
    mut ctxs: EguiContexts,
    _state: ResMut<NextState<EditorState>>,
    mut changes: EventWriter<NewChange>,
    mut state: ResMut<HierarchyTabState>,
    ui_reg: Res<BundleReg>,
) {
    let ctx = ctxs.ctx_mut();
    egui::TopBottomPanel::bottom("bottom_menu").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            let stl = ui.style_mut();
            stl.spacing.button_padding = egui::Vec2::new(8., 2.);

            if ui
                .add(
                    delete_entity_icon(16., 16., "")
                        .stroke(Stroke::new(1., Color32::from_rgb(70, 70, 70))),
                )
                .on_hover_text("Clear all entities")
                .clicked()
            {
                for (entity, _, _, _parent) in query.iter() {
                    commands.entity(entity).despawn_recursive();

                    changes.send(NewChange {
                        change: Arc::new(RemovedEntity { entity }),
                    });
                }
            }
            if ui
                .add(
                    add_entity_icon(16., 16., "")
                        .stroke(Stroke::new(1., Color32::from_rgb(70, 70, 70))),
                )
                .on_hover_text("Add new entity")
                .clicked()
            {
                let id = commands.spawn_empty().insert(PrefabMarker).id();
                changes.send(NewChange {
                    change: Arc::new(AddedEntity { entity: id }),
                });
            }
            if ui
                .add(
                    add_bundle_icon(16., 16., "")
                        .stroke(Stroke::new(1., Color32::from_rgb(70, 70, 70))),
                )
                .on_hover_text("Spawnable preset bundles")
                .clicked()
            {
                state.show_spawnable_bundles = !state.show_spawnable_bundles;
            }
            ui.checkbox(&mut state.show_editor_entities, "Show editor entities");
            if state.show_spawnable_bundles {
                ui.vertical(|ui| {
                    for (category_name, category_bundle) in ui_reg.bundles.iter() {
                        ui.menu_button(category_name, |ui| {
                            let mut categories_vec: Vec<(&String, &EditorBundleUntyped)> =
                                category_bundle.iter().collect();
                            categories_vec.sort_by(|a, b| a.0.cmp(b.0));

                            for (name, dyn_bundle) in categories_vec {
                                if ui.button(name).clicked() {
                                    let entity = dyn_bundle.spawn(&mut commands);
                                    changes.send(NewChange {
                                        change: Arc::new(AddedEntity { entity }),
                                    });
                                }
                            }
                        });
                        ui.separator();
                    }
                });
            }
        });
    });
}

pub fn top_menu(
    mut ctxs: EguiContexts,
    _state: ResMut<NextState<EditorState>>,
    mut events: EventReader<MenuLoadEvent>,
    mut menu_state: ResMut<BottomMenuState>,
    mut editor_events: EventWriter<EditorEvent>,
    background_tasks: Res<BackgroundTaskStorage>,
) {
    let ctx = ctxs.ctx_mut();
    egui::TopBottomPanel::top("top_menu_bar")
        .exact_height(28.)
        .show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                let stl = ui.style_mut();
                stl.spacing.button_padding = egui::Vec2::new(8., 2.);

                if ui.button("📂").clicked() {
                    let mut dialog = egui_file::FileDialog::open_file(Some("assets/".into()))
                        .show_files_filter(Box::new(|path| {
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
                                editor_events.send(EditorEvent::Load(EditorPrefabPath::File(
                                    format!("{}.scn.ron", menu_state.path.clone()),
                                )));
                            }
                        }
                    } else {
                        let mut need_move_to_default_dir = false;
                        if let Some(path) = dialog.directory().to_str() {
                            if !path.contains("assets") {
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

                                editor_events.send(EditorEvent::LoadGltfAsPrefab(path));
                            }
                        }
                    } else {
                        let mut need_move_to_default_dir = false;
                        if let Some(path) = gltf_dialog.directory().to_str() {
                            if !path.contains("assets") {
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

                ui.label("Scene save path:");
                ui.add(egui::TextEdit::singleline(&mut menu_state.path));
                if ui.button("Save scene").clicked() {
                    editor_events.send(EditorEvent::Save(EditorPrefabPath::File(format!(
                        "{}.scn.ron",
                        menu_state.path.clone()
                    ))));
                }

                if ui.button("Load scene").clicked() && !menu_state.path.is_empty() {
                    editor_events.send(EditorEvent::Load(EditorPrefabPath::File(format!(
                        "{}.scn.ron",
                        menu_state.path.clone()
                    ))));
                    // load_server.scene = Some(
                    //     assets.load(format!("{}.scn.ron",save_confg.path))
                    // );
                }

                if ui.button("Open gltf prefab").clicked() {
                    let mut gltf_dialog = egui_file::FileDialog::open_file(Some("assets/".into()))
                        .show_files_filter(Box::new(|path| {
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

                ui.spacing();
                if ui.button("▶").clicked() {
                    editor_events.send(EditorEvent::StartGame);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    if !background_tasks.tasks.is_empty() {
                        //Spinning circle
                        ui.spinner();

                        match &background_tasks.tasks[0] {
                            BackgroundTask::AssetLoading(path, _) => {
                                ui.label(format!("Loading {}", path));
                            }
                            BackgroundTask::None => {}
                        }
                    }
                });
            });
        });

    for event in events.read() {
        menu_state.path = event.path.clone();
        editor_events.send(EditorEvent::Load(EditorPrefabPath::File(format!(
            "{}.scn.ron",
            menu_state.path.clone()
        ))));
    }
    events.clear();
}
