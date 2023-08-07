pub mod ui_reflect;
pub mod refl_impl;

use std::any::TypeId;

use bevy::{prelude::*, ecs::{component::ComponentId, change_detection::MutUntyped}, reflect::ReflectFromPtr, ptr::PtrMut, render::camera::CameraProjection};

use bevy_egui::*;

use egui_gizmo::*;

use crate::{editor_registry::{EditorRegistryExt, EditorRegistry}, EditorSet};

use super::{selected::{SelectedPlugin, Selected}, reset_pan_orbit_state, PanOrbitEnabled, ui_camera_block};
use ui_reflect::*;

#[derive(Component)]
pub struct SkipInspector;


pub struct InspectorPlugin;

impl Plugin for InspectorPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<SelectedPlugin>() {
            app.add_plugins(SelectedPlugin);
        }

        app.init_resource::<InspectState>();

        app.editor_custom_reflect(refl_impl::reflect_name);
        app.editor_custom_reflect::<String, _>(refl_impl::reflect_string);

        app.add_systems(Update, (inspect, execute_inspect_command).chain()
            .after(reset_pan_orbit_state)
            .before(ui_camera_block)
            .in_set(EditorSet::Editor));
    }
}

pub fn mut_untyped_split<'a>(mut mut_untyped: MutUntyped<'a>) -> (PtrMut<'a>, impl FnMut() + 'a) {
    // bypass_change_detection returns a `&mut PtrMut` which is basically useless, because all its methods take `self`
    let ptr = mut_untyped.bypass_change_detection();
    // SAFETY: this is exactly the same PtrMut, just not in a `&mut`. The old one is no longer accessible
    let ptr = unsafe { PtrMut::new(std::ptr::NonNull::new_unchecked(ptr.as_ptr())) };

    (ptr, move || mut_untyped.set_changed())
}

#[derive(Resource)]
struct InspectState {
    create_component_type : Option<ComponentId>,
    commands : Vec<InspectCommand>,
    gizmo_mode : GizmoMode
}

impl Default for InspectState {
    fn default() -> Self {
        Self {
            create_component_type : None,
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

pub fn inspect(
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

    let mut components_id = vec![];
    let mut types_id = vec![];

    let cam_proj;
    let cam_pos;
    {
        let mut cam_query = world.query::<(&Projection, &GlobalTransform)>();
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

        let mut ctx = cell.get_entity(ctx_e).unwrap().get_mut::<EguiContext>().unwrap();
        let mut commands : Vec<InspectCommand> = vec![];
        egui::SidePanel::right("Inspector").show(ctx.get_mut(), |ui| {


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
                        for idx in 0..components_id.len() {
                            let c_id = components_id[idx];
                            let t_id = types_id[idx];
                            if let Some(data) = e.get_mut_by_id(c_id) {
                                let registration = registry
                                    .get(t_id).unwrap();
                                if let Some(reflect_from_ptr) = registration.data::<ReflectFromPtr>() {
                                    let (ptr, mut set_changed) = mut_untyped_split(data);
        
                                    let value = reflect_from_ptr.as_reflect_ptr_mut(ptr);
        
                                    ui.push_id(format!("{}-{}", &name, &registration.short_name()), |ui| {
                                        bevy_inspector_egui::reflect_inspector::ui_for_value(value, ui, &world_registry);
                                    });
                                    // ui_for_reflect(ui, value, &name, registration.short_name(),&mut set_changed, &mut cell);
                                }
                            }
                        }

                        ui.separator();
                        //add component
                        let selected_name;
                        if let Some(selected_id) = state.create_component_type {
                            let selected_info = cell.components().get_info(selected_id).unwrap();
                            selected_name = registry.get(selected_info.type_id().unwrap()).unwrap().short_name().to_string();
                        } else {
                            selected_name = "Press to select".to_string();
                        }
                        let combo = egui::ComboBox::new(format!("inspect_select"), "New")
                            .selected_text(&selected_name).show_ui(ui, |ui| {
                                for idx in 0..components_id.len() {
                                    let c_id = components_id[idx];
                                    let t_id = types_id[idx];

                                    if editor_registry.silent.contains(&t_id) {
                                        continue;
                                    }
                                    
                                    let name = registry.get(t_id).unwrap().short_name();
                                    ui.selectable_value(
                                        &mut state.create_component_type, 
                                        Some(c_id),
                                        name);
                                }
                            });
                        if ui.button("Add component").clicked() {
                            info!("adding component button clicked");
                            if let Some(c_id) = state.create_component_type {
                                info!("adding component {:?}", c_id);
                                let id = cell.components().get_info(c_id).unwrap().type_id().unwrap();
                                commands.push(InspectCommand::AddComponent(e.id(), id));
                            }
                        }
                        if ui.button("Delete component").clicked() {
                            if let Some(c_id) = state.create_component_type {
                                info!("removing component {:?}", c_id);
                                let id = cell.components().get_info(c_id).unwrap().type_id().unwrap();
                                commands.push(InspectCommand::RemoveComponent(e.id(), id));
                            }
                        }
                        
                    }
                }
            });

            let view_matrix = Mat4::from(cam_pos.affine().inverse());

            for e in &selected {
                let Some(ecell) = cell.get_entity(*e) else {
                    continue;
                };
                let Some(mut transform) = ecell.get_mut::<Transform>() else {
                    continue;
                };
                if let Some(parent) = ecell.get::<Parent>() {
                    if let Some(parent) = cell.get_entity(parent.get()) {
                        if let Some(parent_global) = parent.get::<GlobalTransform>() {
                            if let Some(global) = ecell.get::<GlobalTransform>() {
                                if let Some(result) = egui_gizmo::Gizmo::new(format!("Selected gizmo {:?}", *e))
                                    .projection_matrix(cam_proj.get_projection_matrix().to_cols_array_2d())
                                    .view_matrix(view_matrix.to_cols_array_2d())
                                    .model_matrix(global.compute_matrix().to_cols_array_2d())
                                    .mode(state.gizmo_mode.clone())
                                    .interact(ui) {
                                    
                                    let new_transform = Transform {
                                        translation: Vec3::from(<[f32; 3]>::from(result.translation)),
                                        rotation: Quat::from_array(<[f32; 4]>::from(result.rotation)),
                                        scale: Vec3::from(<[f32; 3]>::from(result.scale)),
                                    };
                                    let new_transform = GlobalTransform::from(new_transform);
                                    *transform = new_transform.reparented_to(&parent_global);
                                    transform.set_changed();
                                    
                                }
                                disable_pan_orbit = true;
                                continue;
                            }
                        }
                    }
                }
                if let Some(result) = egui_gizmo::Gizmo::new(format!("Selected gizmo {:?}", *e))
                    .projection_matrix(cam_proj.get_projection_matrix().to_cols_array_2d())
                    .view_matrix(view_matrix.to_cols_array_2d())
                    .model_matrix(transform.compute_matrix().to_cols_array_2d())
                    .mode(state.gizmo_mode.clone())
                    .interact(ui) {
                    *transform = Transform {
                        translation: Vec3::from(<[f32; 3]>::from(result.translation)),
                        rotation: Quat::from_array(<[f32; 4]>::from(result.rotation)),
                        scale: Vec3::from(<[f32; 3]>::from(result.scale)),
                    };
                    transform.set_changed();
                    disable_pan_orbit = true;
                }
                
            }
        });

        state.commands = commands;
    }
    

    if disable_pan_orbit {
        world.resource_mut::<PanOrbitEnabled>().0 = false;
    }
}
