use std::sync::Arc;

use bevy::prelude::*;
use bevy_egui_next::{
    egui::{Align, Align2, Margin, Pos2, Stroke, Widget},
    *,
};
use space_editor_core::prelude::*;
use space_prefab::plugins::PrefabPlugin;
use space_shared::{ext::egui_file, *};
use space_undo::{AddedEntity, NewChange, RemovedEntity};

use crate::{
    colors::*,
    hierarchy::{HierarchyQueryIter, HierarchyTabState},
    icons::{add_bundle_icon, add_entity_icon, delete_entity_icon, prefab_icon},
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
        app.init_resource::<MenuToolbarState>();

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
            if !time.is_paused() {
                *smoothed_dt = (*smoothed_dt).mul_add(0.98, time.delta_seconds() * 0.02);
            }
            let layout = egui::Layout::left_to_right(Align::Center).with_main_align(Align::Center);
            ui.with_layout(layout, |ui| {
                ui.label(format!("FPS: {:04.0}", 1.0 / *smoothed_dt));
                let distance = ui.available_width() / 2. - 64.;
                ui.add_space(distance);
                let button = if time.is_paused() { "▶" } else { "⏸" };
                if ui.button(button).clicked() {
                    if time.is_paused() {
                        time.unpause();
                    } else {
                        time.pause();
                    }
                }
                if ui.button("⏹").clicked() {
                    state.set(EditorState::Editor);
                }
            });
        });
}

#[derive(Resource, Default)]
pub struct MenuToolbarState {
    pub file_dialog: Option<egui_file::FileDialog>,
    pub gltf_dialog: Option<egui_file::FileDialog>,
    pub save_dialog: Option<egui_file::FileDialog>,
    pub load_dialog: Option<egui_file::FileDialog>,
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
    menu_state: Res<MenuToolbarState>,
) {
    let ctx = ctxs.ctx_mut();
    egui::TopBottomPanel::bottom("bottom_menu").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            let stl = ui.style_mut();
            stl.spacing.button_padding = egui::Vec2::new(8., 2.);

            if ui
                .add(delete_entity_icon(16., 16., "").stroke(stroke_default_color()))
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
                .add(add_entity_icon(16., 16., "").stroke(stroke_default_color()))
                .on_hover_text("Add new entity")
                .clicked()
            {
                let id = commands.spawn_empty().insert(PrefabMarker).id();
                changes.send(NewChange {
                    change: Arc::new(AddedEntity { entity: id }),
                });
            }
            let spawnable_button = add_bundle_icon(16., 16., "").stroke(stroke_default_color());

            let spawnables = ui.add(if state.show_spawnable_bundles {
                spawnable_button.fill(SELECTED_ITEM_COLOR)
            } else {
                spawnable_button
            });
            let spawnable_pos = Pos2 {
                x: 16.,
                y: spawnables.rect.right_top().y - 4.,
            };
            if spawnables
                .on_hover_text("Spawnable preset bundles")
                .clicked()
            {
                state.show_spawnable_bundles = !state.show_spawnable_bundles;
            }

            if state.show_spawnable_bundles {
                egui::Window::new("Bundles")
                    .frame(
                        egui::Frame::none()
                            .inner_margin(Margin::symmetric(8., 4.))
                            .rounding(3.)
                            .stroke(stroke_default_color())
                            .fill(SPECIAL_BG_COLOR),
                    )
                    .collapsible(false)
                    .pivot(Align2::LEFT_BOTTOM)
                    .default_pos(spawnable_pos)
                    .default_size(egui::Vec2::new(80., 80.))
                    .title_bar(false)
                    .show(ctx, |ui| {
                        egui::menu::bar(ui, |ui| {
                            ui.spacing();
                            for (category_name, category_bundle) in ui_reg.bundles.iter() {
                                ui.menu_button(category_name, |ui| {
                                    let mut categories_vec: Vec<(&String, &EditorBundleUntyped)> =
                                        category_bundle.iter().collect();
                                    categories_vec.sort_by(|a, b| a.0.cmp(b.0));

                                    for (name, dyn_bundle) in categories_vec {
                                        let button = egui::Button::new(name).ui(ui);
                                        if button.clicked() {
                                            let entity = dyn_bundle.spawn(&mut commands);
                                            changes.send(NewChange {
                                                change: Arc::new(AddedEntity { entity }),
                                            });
                                        }
                                    }
                                });
                                ui.add_space(32.);
                            }
                            if ui.button("🗙").clicked() {
                                state.show_spawnable_bundles = !state.show_spawnable_bundles;
                            }
                        });
                    });
            }
            ui.spacing();
            ui.checkbox(&mut state.show_editor_entities, "Show editor entities");
            let distance = ui.available_width() * 0.66;
            ui.add_space(distance);
            ui.label(format!("Current Scene: {}", menu_state.path));
        });
    });
}

pub fn top_menu(
    mut ctxs: EguiContexts,
    _state: ResMut<NextState<EditorState>>,
    mut events: EventReader<MenuLoadEvent>,
    mut menu_state: ResMut<MenuToolbarState>,
    mut editor_events: EventWriter<EditorEvent>,
    background_tasks: Res<BackgroundTaskStorage>,
) {
    let ctx = ctxs.ctx_mut();
    egui::TopBottomPanel::top("top_menu_bar")
        .exact_height(28.)
        .show(ctx, |ui| {
            ui.style_mut().spacing.menu_margin = Margin::symmetric(16., 4.);
            egui::menu::bar(ui, |ui| {
                let stl = ui.style_mut();
                stl.spacing.button_padding = egui::Vec2::new(8., 4.);

                // Open Assets Folder
                let open_button = egui::Button::new("📂").stroke(stroke_default_color());
                if ui.add(open_button).clicked() {
                    let mut dialog = egui_file::FileDialog::open_file(Some("assets/".into()))
                        .show_files_filter(Box::new(|path| {
                            path.to_str().unwrap().ends_with(".scn.ron")
                        }))
                        .title("File Explorer (Scene/Bundle) (*.scn.ron)");
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
                // END Open Assets Folder

                // Save file
                let file_button = egui::Button::new("💾").stroke(stroke_default_color());
                if ui
                    .add(file_button)
                    .on_hover_text("Save current scene")
                    .clicked()
                {
                    let mut save_dialog =
                        egui_file::FileDialog::save_file(Some("./assets/scenes".into()))
                            .default_filename("Scene0.scn.ron")
                            .title("Save Scene");
                    save_dialog.open();
                    menu_state.save_dialog = Some(save_dialog);
                }

                if let Some(save_dialog) = &mut menu_state.save_dialog {
                    if save_dialog.show(ctx).selected() {
                        if let Some(file) = save_dialog.path() {
                            let path = file.to_str().unwrap().to_string();
                            //remove assets/ from path
                            if path.ends_with(".scn.ron") {
                                let path = path.replace(".scn.ron", "");
                                println!("{path}");
                                editor_events.send(EditorEvent::Save(EditorPrefabPath::File(
                                    format!("{}.scn.ron", path),
                                )));
                            }
                        }
                    } else {
                        let mut need_move_to_default_dir = false;
                        if let Some(path) = save_dialog.directory().to_str() {
                            if !path.contains("assets") {
                                need_move_to_default_dir = true;
                            }
                        } else {
                            need_move_to_default_dir = true;
                        }
                        if need_move_to_default_dir {
                            save_dialog.set_path("assets/");
                        }
                    }
                }
                // End Save File

                // Load Scene
                let load_button = egui::Button::new("📤").stroke(stroke_default_color());
                if ui
                    .add(load_button)
                    .on_hover_text("Load scene file")
                    .clicked()
                {
                    let mut dialog = egui_file::FileDialog::open_file(Some("assets/scenes".into()))
                        .show_files_filter(Box::new(|path| {
                            path.to_str().unwrap().ends_with(".scn.ron")
                        }))
                        .title("Load Scene (*.scn.ron)");
                    dialog.open();
                    menu_state.load_dialog = Some(dialog);
                }

                if let Some(dialog) = &mut menu_state.load_dialog {
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
                // END Load Scene

                // Open GLTF
                let open_gltf_button = prefab_icon(16., 16., "").stroke(stroke_default_color());
                if ui
                    .add(open_gltf_button)
                    .on_hover_text("Open GLTF/GLB as prefab")
                    .clicked()
                {
                    let mut gltf_dialog =
                        egui_file::FileDialog::open_file(Some("assets/models".into()))
                            .show_files_filter(Box::new(|path| {
                                path.to_str().unwrap().ends_with(".gltf")
                                    || path.to_str().unwrap().ends_with(".glb")
                            }))
                            .title("Opens GLTF as Prefab");
                    gltf_dialog.open();
                    menu_state.gltf_dialog = Some(gltf_dialog);
                }

                if let Some(gltf_dialog) = &mut menu_state.gltf_dialog {
                    if gltf_dialog.show(ctx).selected() {
                        if let Some(file) = gltf_dialog.path() {
                            let mut path = file.to_str().unwrap().to_string();
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
                // End Open GLTF

                let distance = ui.available_width() / 2. - 40.;
                ui.add_space(distance);
                let play_button = egui::Button::new("▶")
                    .fill(SPECIAL_BG_COLOR)
                    .stroke(Stroke {
                        width: 1.,
                        color: SELECTED_ITEM_COLOR,
                    });
                if ui.add(play_button).clicked() {
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
