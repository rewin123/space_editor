use std::sync::Arc;

use bevy::prelude::*;
use bevy_egui::{
    egui::{Align, Align2, Margin, Pos2, Stroke, Widget},
    *,
};
use bevy_panorbit_camera::PanOrbitCamera;
use egui_dock::egui::RichText;
use space_editor_core::{
    prelude::*,
    toast::{ClearToastMessage, ToastStorage},
};
use space_editor_tabs::prelude::*;
use space_prefab::{component::GltfPrefab, load::PrefabBundle, plugins::PrefabPlugin};
use space_shared::{ext::egui_file, *};
use space_undo::{AddedEntity, NewChange, RemovedEntity};

use crate::{
    hierarchy::{HierarchyQueryIter, HierarchyTabState},
    icons::{add_bundle_icon, add_entity_icon, delete_entity_icon, prefab_icon},
    sizing::{to_colored_richtext, to_richtext},
    ui_registration::{BundleReg, EditorBundleUntyped},
    ShowEditorUi,
};

use crate::{colors::*, sizing::Sizing};

/// Plugin to activate bottom menu in editor UI
pub struct BottomMenuPlugin;

impl Plugin for BottomMenuPlugin {
    #[cfg(not(tarpaulin_include))]
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<PrefabPlugin>() {
            app.add_plugins(PrefabPlugin);
        }

        app.init_resource::<EditorLoader>();
        app.init_resource::<MenuToolbarState>();

        app.add_systems(
            Update,
            bottom_menu
                .before(EditorLoadSet)
                .in_set(EditorSet::Editor)
                .run_if(in_state(EditorState::Editor).and_then(in_state(ShowEditorUi::Show))),
        );
        app.add_systems(
            Update,
            top_menu
                .before(EditorLoadSet)
                .in_set(EditorSet::Editor)
                .run_if(in_state(EditorState::Editor).and_then(in_state(ShowEditorUi::Show))),
        );
        app.add_systems(Update, in_game_menu.in_set(EditorSet::Game));
        app.add_event::<MenuLoadEvent>();
    }
}

#[derive(Event)]
pub struct MenuLoadEvent {
    pub path: String,
}

pub struct FrameSpeedMultiplier {
    pub ratio: f32,
}

impl Default for FrameSpeedMultiplier {
    fn default() -> Self {
        Self { ratio: 1.0 }
    }
}

fn in_game_menu(
    mut smoothed_dt: Local<f32>,
    mut frame_speed_mult: Local<FrameSpeedMultiplier>,
    mut ctxs: EguiContexts,
    mut state: ResMut<NextState<EditorState>>,
    mut time: ResMut<Time<Virtual>>,
    sizing: Res<Sizing>,
) {
    egui::TopBottomPanel::top("top_gameplay_panel")
        .min_height(&sizing.icon.to_size() + 8.)
        .show(ctxs.ctx_mut(), |ui| {
            let frame_duration = time.delta();
            if !time.is_paused() {
                *smoothed_dt = (*smoothed_dt).mul_add(0.98, time.delta_seconds() * 0.02);
            }
            let layout = egui::Layout::left_to_right(Align::Center).with_main_align(Align::Center);
            ui.with_layout(layout, |ui| {
                ui.label(format!("FPS: {:04.0}", 1.0 / *smoothed_dt));

                let distance = ui.available_width() / 2. - 64.;
                ui.add_space(distance);
                let button = if time.is_paused() {
                    to_richtext("▶", &sizing.icon)
                } else {
                    to_richtext("⏸", &sizing.icon)
                };
                if ui.button(button).clicked() {
                    if time.is_paused() {
                        time.unpause();
                    } else {
                        time.pause();
                    }
                }
                if ui.button(to_richtext("⏹", &sizing.icon)).clicked() {
                    state.set(EditorState::Editor);
                }
                if ui
                    .button(to_richtext("⏭", &sizing.icon))
                    .on_hover_text("Step by delta time")
                    .clicked()
                {
                    time.advance_by(frame_duration);
                }

                ui.add_space(60.);
                if egui::DragValue::new(&mut frame_speed_mult.ratio)
                    .suffix(" x")
                    .range((0.)..=5.)
                    .speed(1. / 60.)
                    .fixed_decimals(2)
                    .ui(ui)
                    .changed()
                {
                    time.set_relative_speed(frame_speed_mult.ratio);
                };

                if ui
                    .button(to_richtext("⟲", &sizing.icon))
                    .on_hover_text("Reset frame speed multiplier to 1.0 ")
                    .clicked()
                {
                    frame_speed_mult.ratio = 1.;
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
    pub subscene_dialog: Option<egui_file::FileDialog>,
    show_toasts: bool,
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
    sizing: Res<Sizing>,
    q_pan_cam: Query<&PanOrbitCamera>,
) {
    let ctx = ctxs.ctx_mut();
    egui::TopBottomPanel::bottom("bottom_menu")
        .min_height(&sizing.icon.to_size().max(sizing.text) + 4.)
        .show(ctx, |ui| {
            ui.style_mut().spacing.menu_margin = Margin::symmetric(16., 8.);
            egui::menu::bar(ui, |ui| {
                let stl = ui.style_mut();
                stl.spacing.button_padding = egui::Vec2::new(8., 2.);

                if ui
                    .add(
                        delete_entity_icon(sizing.icon.to_size(), "")
                            .stroke(stroke_default_color()),
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
                    .add(add_entity_icon(sizing.icon.to_size(), "").stroke(stroke_default_color()))
                    .on_hover_text("Add new entity")
                    .clicked()
                {
                    let id = commands.spawn_empty().insert(PrefabMarker).id();
                    changes.send(NewChange {
                        change: Arc::new(AddedEntity { entity: id }),
                    });
                }
                let spawnable_button =
                    add_bundle_icon(sizing.icon.to_size(), "").stroke(stroke_default_color());

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
                                        let mut categories_vec: Vec<(
                                            &String,
                                            &EditorBundleUntyped,
                                        )> = category_bundle.iter().collect();
                                        categories_vec.sort_by(|a, b| a.0.cmp(b.0));

                                        for (name, dyn_bundle) in categories_vec {
                                            let button = egui::Button::new(name).ui(ui);
                                            if button.clicked() {
                                                let entity = dyn_bundle.spawn(&mut commands);
                                                if let Ok(pan_cam) = q_pan_cam.get_single() {
                                                    commands.entity(entity).insert(
                                                        SpatialBundle::from_transform(
                                                            Transform::from_translation(
                                                                pan_cam.focus,
                                                            ),
                                                        ),
                                                    );
                                                }
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
                ui.style_mut().spacing.icon_width = sizing.text - 4.;
                ui.checkbox(
                    &mut state.show_editor_entities,
                    to_label("Show editor entities", sizing.text),
                );
                let distance = ui.available_width() * 0.66 * 12. / sizing.text;
                ui.add_space(distance);
                ui.label(to_label(
                    &format!("Current Scene: {}", menu_state.path),
                    sizing.text,
                ));
            });
        });
}

pub fn top_menu(
    mut commands: Commands,
    mut ctxs: EguiContexts,
    _state: ResMut<NextState<EditorState>>,
    mut events: EventReader<MenuLoadEvent>,
    mut menu_state: ResMut<MenuToolbarState>,
    mut editor_events: EventWriter<EditorEvent>,
    mut clear_toast: EventWriter<ClearToastMessage>,
    background_tasks: Res<BackgroundTaskStorage>,
    toasts: Res<ToastStorage>,
    sizing: Res<Sizing>,
) {
    let ctx = ctxs.ctx_mut();
    egui::TopBottomPanel::top("top_menu_bar")
        .min_height(&sizing.icon.to_size() + 8.)
        .show(ctx, |ui| {
            ui.style_mut().spacing.menu_margin = Margin::symmetric(16., 8.);
            egui::menu::bar(ui, |ui| {
                let stl = ui.style_mut();
                stl.spacing.button_padding = egui::Vec2::new(8., 4.);

                // Open Assets Folder
                let open_button = egui::Button::new(to_richtext("📂", &sizing.icon))
                    .stroke(stroke_default_color());
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
                let file_button = egui::Button::new(to_richtext("💾", &sizing.icon))
                    .stroke(stroke_default_color());
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
                let load_button = egui::Button::new(to_richtext("📤", &sizing.icon))
                    .stroke(stroke_default_color());
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
                let open_gltf_button =
                    prefab_icon(sizing.icon.to_size(), "").stroke(stroke_default_color());
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

                //Open subscene
                let subscene_button = egui::Button::new(to_richtext("📦", &sizing.icon))
                    .stroke(stroke_default_color());
                if ui
                    .add(subscene_button)
                    .on_hover_text("Open subscene")
                    .clicked()
                {
                    let mut filedialog = egui_file::FileDialog::open_file(Some("assets".into()))
                        .show_files_filter(Box::new(|path| {
                            path.to_str().unwrap().ends_with(".scn.ron")
                                || path.to_str().unwrap().ends_with(".gltf")
                                || path.to_str().unwrap().ends_with(".glb")
                        }))
                        .title("Open Subscene (.scn.ron, .gltf, .glb)");
                    filedialog.open();

                    menu_state.subscene_dialog = Some(filedialog);
                }

                if let Some(subscene_dialog) = &mut menu_state.subscene_dialog {
                    if subscene_dialog.show(ctx).selected() {
                        if let Some(file) = subscene_dialog.path() {
                            let mut path = file.to_str().unwrap().to_string();
                            info!("path: {}", path);
                            if path.starts_with("assets") {
                                path = path.replace("assets", "");
                                path = path.trim_start_matches('\\').to_string();
                                path = path.trim_start_matches('/').to_string();

                                if path.ends_with(".scn.ron") {
                                    commands.spawn((PrefabBundle::new(&path), PrefabMarker));
                                } else if path.ends_with(".gltf") || path.ends_with(".glb") {
                                    commands.spawn((
                                        SpatialBundle::default(),
                                        GltfPrefab {
                                            path,
                                            scene: "Scene0".into(),
                                        },
                                        PrefabMarker,
                                    ));
                                } else {
                                    error!("Unknown file type: {}", path);
                                }
                            }
                        }
                    }
                }

                let width = ui.available_width();
                let distance = width / 2. - 40.;
                ui.add_space(distance);
                let play_button =
                    egui::Button::new(to_colored_richtext("▶", &sizing.icon, PLAY_COLOR))
                        .fill(SPECIAL_BG_COLOR)
                        .stroke(Stroke {
                            width: 1.,
                            color: STROKE_COLOR,
                        });
                if ui.add(play_button).clicked() {
                    editor_events.send(EditorEvent::StartGame);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::RIGHT), |ui| {
                    if toasts.has_toasts() {
                        egui::Window::new("Errors")
                            .default_size(egui::Vec2::new(80., 32.))
                            .default_pos(egui::pos2(width, 32.))
                            .movable(true)
                            .resizable(true)
                            .open(&mut menu_state.show_toasts)
                            .show(ctx, |ui| {
                                ui.vertical_centered_justified(|ui| {
                                    if ui.add(egui::Button::new("Clear all 🗑")).clicked() {
                                        clear_toast.send(ClearToastMessage::all());
                                    };
                                });
                                egui::Grid::new("error_console_log").show(ui, |ui| {
                                    for (index, error) in
                                        toasts.toasts_per_kind.error.iter().enumerate()
                                    {
                                        ui.label(RichText::new("ERROR").color(ERROR_COLOR));
                                        ui.label(error);
                                        if ui.button("🗙").clicked() {
                                            clear_toast.send(ClearToastMessage::error(index));
                                        }
                                        ui.end_row();
                                    }
                                    for (index, warning) in
                                        toasts.toasts_per_kind.warning.iter().enumerate()
                                    {
                                        ui.label(RichText::new("WARN ").color(WARN_COLOR));
                                        ui.label(warning);
                                        if ui.button("🗙").clicked() {
                                            clear_toast.send(ClearToastMessage::warn(index));
                                        }
                                        ui.end_row();
                                    }
                                })
                            });
                    }
                    if ui
                        .button(
                            RichText::new(format!("⚠ {}", toasts.toasts_per_kind.warning.len()))
                                .color(if toasts.has_toasts() {
                                    WARN_COLOR
                                } else {
                                    STROKE_COLOR
                                }),
                        )
                        .clicked()
                    {
                        menu_state.show_toasts = !menu_state.show_toasts;
                    }
                    if ui
                        .button(
                            RichText::new(format!("🚫 {}", toasts.toasts_per_kind.error.len()))
                                .color(if toasts.has_toasts() {
                                    ERROR_COLOR
                                } else {
                                    STROKE_COLOR
                                }),
                        )
                        .clicked()
                    {
                        menu_state.show_toasts = !menu_state.show_toasts;
                    }

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
        menu_state.path.clone_from(&event.path);
        editor_events.send(EditorEvent::Load(EditorPrefabPath::File(format!(
            "{}.scn.ron",
            menu_state.path.clone()
        ))));
    }
    events.clear();
}
