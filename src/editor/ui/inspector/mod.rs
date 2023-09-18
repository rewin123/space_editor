pub mod refl_impl;

use std::any::TypeId;

use bevy::{prelude::*, ecs::{component::ComponentId, change_detection::MutUntyped, system::CommandQueue}, reflect::ReflectFromPtr, ptr::PtrMut, render::camera::CameraProjection, utils::HashMap};

use bevy_egui::*;

use bevy_inspector_egui::{reflect_inspector::InspectorUi, inspector_egui_impls::InspectorEguiImpl};
use egui_gizmo::*;

use crate::{editor_registry::{EditorRegistryExt, EditorRegistry}, EditorSet, EditorCameraMarker, PrefabSet, prefab::component::EntityLink, prelude::{SelectedPlugin, Selected, EditorTab}};

use self::refl_impl::{entity_ref_ui, entity_ref_ui_readonly, many_unimplemented};

use super::EditorUiAppExt;

/// Entities with this marker will be skiped in inspector
#[derive(Component)]
pub struct SkipInspector;

/// Plugin to activate components inspector and gizmo in editor UI
pub struct SpaceInspectorPlugin;

impl Plugin for SpaceInspectorPlugin {
    fn build(&self, app: &mut App) {
        

        app.init_resource::<InspectState>();

        app.editor_tab_by_trait(crate::prelude::EditorTabName::Inspector, InspectorTab::default());
        // app.add_systems(Update, (inspect, execute_inspect_command).chain()
        //     .after(crate::editor::reset_pan_orbit_state)
        //     .before(crate::editor::ui_camera_block)
        //     .in_set(EditorSet::Editor).before(PrefabSet::DetectPrefabChange));

        app.add_systems(Update, execute_inspect_command);

        app.add_systems(Startup, register_custom_impls);
    }
}

#[derive(Resource, Default)]
pub struct InspectorTab {

}

impl EditorTab for InspectorTab {
    fn ui(&mut self, ui : &mut egui::Ui, world : &mut World) {
        inspect(ui, world);
    }

    fn title(&self) -> egui::WidgetText {
        "Inspector".into()
    }
}

fn register_custom_impls(
    registry : Res<AppTypeRegistry>
) {
    let mut registry = registry.write();
    registry.get_mut(TypeId::of::<EntityLink>())
        .unwrap_or_else(|| panic!("{} not registered", std::any::type_name::<EntityLink>()))
        .insert(
            InspectorEguiImpl::new(
                entity_ref_ui,
                entity_ref_ui_readonly,
                many_unimplemented::<EntityRef>
            )
        );
}

/// Function form bevy_inspector_egui to split component to data ptr and "set changed" function
pub fn mut_untyped_split<'a>(mut mut_untyped: MutUntyped<'a>) -> (PtrMut<'a>, impl FnMut() + 'a) {
    // bypass_change_detection returns a `&mut PtrMut` which is basically useless, because all its methods take `self`
    let ptr = mut_untyped.bypass_change_detection();
    // SAFETY: this is exactly the same PtrMut, just not in a `&mut`. The old one is no longer accessible
    let ptr = unsafe { PtrMut::new(std::ptr::NonNull::new_unchecked(ptr.as_ptr())) };

    (ptr, move || mut_untyped.set_changed())
}

/// Just state of inspector panel
#[derive(Resource)]
struct InspectState {
    component_add_filter : String,
    commands : Vec<InspectCommand>,
    gizmo_mode : GizmoMode
}

impl Default for InspectState {
    fn default() -> Self {
        Self {
            component_add_filter : "".to_string(),
            commands : vec![],
            gizmo_mode : GizmoMode::Translate
        }
    }
}


enum InspectCommand {
    AddComponent(Entity, TypeId),
    RemoveComponent(Entity, TypeId)
}

fn execute_inspect_command(
    mut commands : Commands,
    mut state : ResMut<InspectState>,
    registration : Res<EditorRegistry>
) {
    for c in &state.commands {
        match c {
            InspectCommand::AddComponent(e, id) => {
                info!("inspector adding component {:?} to entity {:?}", id, e);
                commands.entity(*e).add(registration.get_spawn_command(id));
            },
            InspectCommand::RemoveComponent(e, id) => {
                registration.remove_by_id(&mut commands.entity(*e), id);
            },
        }
    }
    state.commands.clear();
}


/// System to show inspector panel
pub fn inspect(
    ui : &mut egui::Ui,
    world : &mut World
) {

    let selected = world.query_filtered::<Entity, With<Selected>>()
        .iter(&world)
        .collect::<Vec<_>>();

    let editor_registry = world.resource::<EditorRegistry>().clone();
    let all_registry = editor_registry.registry.clone();
    let registry = all_registry.read();
    let app_registry = world.resource::<AppTypeRegistry>().clone();
    let world_registry = app_registry.read();
    let mut disable_pan_orbit = false;
    let ctx_e;
    {
        let mut ctx_query = world.query_filtered::<Entity, (With<EguiContext>, With<Window>)>();
        ctx_e = ctx_query.get_single(&world).unwrap();
    }

    let mut components_id = Vec::new();
    let mut types_id = Vec::new();
    let mut components_by_entity: HashMap<u32, Vec<ComponentId>> = HashMap::new();
    let mut types_by_entity: HashMap<u32, Vec<TypeId>> = HashMap::new();

    let cam_proj;
    let cam_pos;
    {
        let mut cam_query = world.query_filtered::<(&Projection, &GlobalTransform), With<EditorCameraMarker>>();
        let (proj, pos) = cam_query.single(world);
        cam_proj = proj.clone();
        cam_pos = pos.clone();
    }
    for reg in registry.iter() {
        if let Some(c_id) = world.components().get_id(reg.type_id()) {
            components_id.push(c_id);
            types_id.push(reg.type_id());
        }
    }

    unsafe {
        let mut cell = world.as_unsafe_world_cell();
        let mut state = cell.get_resource_mut::<InspectState>().unwrap();

        let mut commands : Vec<InspectCommand> = vec![];
            

            egui::ComboBox::new("gizmo_mode", "Gizmo mode").selected_text(format!("{:?}", &state.gizmo_mode))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut state.gizmo_mode, GizmoMode::Translate, format!("{:?}", GizmoMode::Translate));
                    ui.selectable_value(
                        &mut state.gizmo_mode, GizmoMode::Rotate, format!("{:?}", GizmoMode::Rotate));
                    ui.selectable_value(
                        &mut state.gizmo_mode, GizmoMode::Scale, format!("{:?}", GizmoMode::Scale));
                });

            ui.label("Press T, R, F to select Translate/Rotate/Scale mode");
            if ui.input(|i| i.key_pressed(egui::Key::T)) {
                state.gizmo_mode = GizmoMode::Translate;
            }
            if ui.input(|i| i.key_pressed(egui::Key::R)) {
                state.gizmo_mode = GizmoMode::Rotate;
            }
            if ui.input(|i| i.key_pressed(egui::Key::F)) {
                state.gizmo_mode = GizmoMode::Scale;
            }

            ui.separator();


            let mut queue = CommandQueue::default();
            let mut cx = bevy_inspector_egui::reflect_inspector::Context {
                world: Some(cell.world_mut().into()),
                queue: Some(&mut queue),
            };
            let mut env = InspectorUi::for_bevy(&world_registry, &mut cx);

            egui::ScrollArea::vertical().show(ui, |ui| {
                for e in selected.iter() {
                    if let Some(e) = cell.get_entity(*e) {
                        let mut name;
                        if let Some(name_struct) = e.get::<Name>() {
                            name = name_struct.as_str().to_string();
                            if name == "" {
                                name = format!("{:?} (empty name)", e.id());
                            }
                         } else {
                            name = format!("{:?}", e.id());
                        }
                        ui.heading(&name);
                        ui.label("Components:");
                        let e_id = e.id().index();
                        egui::Grid::new(format!("{e_id}")).show(ui, |ui| {
                            for idx in 0..components_id.len() {
                                let c_id = components_by_entity
                                    .entry(e_id)
                                    .or_insert(Vec::new())
                                    .get(idx)
                                    .map(|v| *v)
                                    .unwrap_or(components_id[idx]);
                                let t_id = types_by_entity
                                    .entry(e_id)
                                    .or_insert(Vec::new())
                                    .get(idx)
                                    .map(|v| *v)
                                    .unwrap_or(types_id[idx]);
                                
                                if let Some(data) = e.get_mut_by_id(c_id) {
                                    let registration = registry
                                        .get(t_id).unwrap();
                                    if let Some(reflect_from_ptr) = registration.data::<ReflectFromPtr>() {
                                        let (ptr, mut set_changed) = mut_untyped_split(data);
            
                                        let value = reflect_from_ptr.as_reflect_ptr_mut(ptr);
    
                                        if !editor_registry.silent.contains(&registration.type_id()) {
                                            ui.push_id(format!("{:?}-{}", &e.id(), &registration.short_name()), |ui| {
                                                ui.collapsing(registration.short_name(), |ui| {
                                                    ui.push_id(format!("content-{:?}-{}", &e.id(), &registration.short_name()), |ui| {
                                                        if env.ui_for_reflect_with_options(value, ui, ui.id(), &()) {
                                                            set_changed();
                                                        }
                                                    });
                                                });
                                            });

                                            ui.push_id(format!("del component {:?}-{}", &e.id(), &registration.short_name()), |ui| {
                                                //must be on top
                                                ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
                                                    if ui.button("X").clicked() {
                                                        commands.push(InspectCommand::RemoveComponent(e.id(), t_id));
                                                    }
                                                });
                                            });
                                            ui.end_row();
                                        }
                                    }
                                }

                            }
                        });
                        
                        ui.separator();
                    }
                }

                ui.label("Add component");
                ui.text_edit_singleline(&mut state.component_add_filter);
                let lower_filter = state.component_add_filter.to_lowercase();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    egui::Grid::new("Component grid").show(ui, |ui| {
                        let mut counter = 0;
                        for idx in 0..components_id.len() {
                            let c_id = components_id[idx];
                            let t_id = types_id[idx];
                            let name = registry.get(t_id).unwrap().short_name();

                            if name.to_lowercase().contains(&lower_filter) {
                                ui.label(name);
                                if ui.button("+").clicked() {
                                    let id = cell.components().get_info(c_id).unwrap().type_id().unwrap();
                                    for e in selected.iter() {
                                        commands.push(InspectCommand::AddComponent(*e, id));
                                    }
                                }
                                ui.end_row();
                            }
                        }
                    });
                });
        });

        state.commands = commands;
    }
    

    if disable_pan_orbit {
        world.resource_mut::<crate::editor::PanOrbitEnabled>().0 = false;
    }
}
