pub mod refl_impl;

use std::any::TypeId;

use bevy::{
    ecs::{change_detection::MutUntyped, component::ComponentId, system::CommandQueue},
    prelude::*,
    ptr::PtrMut,
    reflect::ReflectFromPtr,
    utils::HashMap,
};

use bevy_egui::*;

use bevy_inspector_egui::{
    inspector_egui_impls::InspectorEguiImpl, reflect_inspector::InspectorUi,
};

use crate::{
    editor::core::Selected, editor_registry::EditorRegistry, prefab::component::EntityLink,
    prelude::EditorTab,
};

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

        app.editor_tab_by_trait(
            crate::prelude::EditorTabName::Inspector,
            InspectorTab::default(),
        );
        // app.add_systems(Update, (inspect, execute_inspect_command).chain()
        //     .after(crate::editor::reset_pan_orbit_state)
        //     .before(crate::editor::ui_camera_block)
        //     .in_set(EditorSet::Editor).before(PrefabSet::DetectPrefabChange));

        app.add_systems(Update, execute_inspect_command);

        app.add_systems(Startup, register_custom_impls);
    }
}

#[derive(Resource, Default)]
pub struct InspectorTab {}

impl EditorTab for InspectorTab {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World) {
        inspect(ui, world);
    }

    fn title(&self) -> egui::WidgetText {
        "Inspector".into()
    }
}

fn register_custom_impls(registry: Res<AppTypeRegistry>) {
    let mut registry = registry.write();
    registry
        .get_mut(TypeId::of::<EntityLink>())
        .unwrap_or_else(|| panic!("{} not registered", std::any::type_name::<EntityLink>()))
        .insert(InspectorEguiImpl::new(
            entity_ref_ui,
            entity_ref_ui_readonly,
            many_unimplemented::<EntityRef>,
        ));
}

/// Function form `bevy_inspector_egui` to split component to data ptr and "set changed" function
pub fn mut_untyped_split(mut mut_untyped: MutUntyped<'_>) -> (PtrMut<'_>, impl FnMut() + '_) {
    // bypass_change_detection returns a `&mut PtrMut` which is basically useless, because all its methods take `self`
    let ptr = mut_untyped.bypass_change_detection();
    // SAFETY: this is exactly the same PtrMut, just not in a `&mut`. The old one is no longer accessible
    let ptr = unsafe { PtrMut::new(std::ptr::NonNull::new_unchecked(ptr.as_ptr())) };

    (ptr, move || mut_untyped.set_changed())
}

/// Just state of inspector panel
#[derive(Resource)]
struct InspectState {
    component_add_filter: String,
    commands: Vec<InspectCommand>,
}

impl Default for InspectState {
    fn default() -> Self {
        Self {
            component_add_filter: "".to_string(),
            commands: vec![],
        }
    }
}

enum InspectCommand {
    AddComponent(Entity, TypeId),
    RemoveComponent(Entity, TypeId),
}

fn execute_inspect_command(
    mut commands: Commands,
    mut state: ResMut<InspectState>,
    registration: Res<EditorRegistry>,
) {
    for c in &state.commands {
        match c {
            InspectCommand::AddComponent(e, id) => {
                info!("inspector adding component {:?} to entity {:?}", id, e);
                commands.entity(*e).add(registration.get_spawn_command(id));
            }
            InspectCommand::RemoveComponent(e, id) => {
                registration.remove_by_id(&mut commands.entity(*e), id);
            }
        }
    }
    state.commands.clear();
}

/// System to show inspector panel
pub fn inspect(ui: &mut egui::Ui, world: &mut World) {
    let selected = world
        .query_filtered::<Entity, With<Selected>>()
        .iter(world)
        .collect::<Vec<_>>();

    let editor_registry = world.resource::<EditorRegistry>().clone();
    let all_registry = editor_registry.registry.clone();
    let registry = all_registry.read();
    let app_registry = world.resource::<AppTypeRegistry>().clone();
    let world_registry = app_registry.read();
    let disable_pan_orbit = false;

    let mut components_id = Vec::new();
    let mut types_id = Vec::new();
    let mut components_by_entity: HashMap<u32, Vec<ComponentId>> = HashMap::new();
    let mut types_by_entity: HashMap<u32, Vec<TypeId>> = HashMap::new();

    for reg in registry.iter() {
        if let Some(c_id) = world.components().get_id(reg.type_id()) {
            components_id.push(c_id);
            types_id.push(reg.type_id());
        }
    }

    unsafe {
        let cell = world.as_unsafe_world_cell();
        let mut state = cell.get_resource_mut::<InspectState>().unwrap();

        let mut commands: Vec<InspectCommand> = vec![];
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
                        if name.is_empty() {
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
                            let c_id: ComponentId = components_by_entity
                                .entry(e_id)
                                .or_insert(Vec::new())
                                .get(idx)
                                .copied()
                                .unwrap_or(components_id[idx]);
                            let t_id = types_by_entity
                                .entry(e_id)
                                .or_insert(Vec::new())
                                .get(idx)
                                .copied()
                                .unwrap_or(types_id[idx]);

                            if let Some(data) = e.get_mut_by_id(c_id) {
                                let registration = registry.get(t_id).unwrap();
                                if let Some(reflect_from_ptr) =
                                    registration.data::<ReflectFromPtr>()
                                {
                                    let (ptr, mut set_changed) = mut_untyped_split(data);

                                    let value = reflect_from_ptr.from_ptr_mut()(ptr);

                                    let name = {
                                        let info = cell.components().get_info(c_id).unwrap();
                                        pretty_type_name::pretty_type_name_str(info.name())
                                    };

                                    if !editor_registry.silent.contains(&registration.type_id()) {
                                        ui.push_id(format!("{:?}-{}", &e.id(), &name), |ui| {
                                            ui.collapsing(&name, |ui| {
                                                ui.push_id(
                                                    format!("content-{:?}-{}", &e.id(), &name),
                                                    |ui| {
                                                        if env.ui_for_reflect_with_options(
                                                            value,
                                                            ui,
                                                            ui.id(),
                                                            &(),
                                                        ) {
                                                            set_changed();
                                                        }
                                                    },
                                                );
                                            });
                                        });

                                        ui.push_id(
                                            format!("del component {:?}-{}", &e.id(), &name),
                                            |ui| {
                                                //must be on top
                                                ui.with_layout(
                                                    egui::Layout::top_down(egui::Align::Min),
                                                    |ui| {
                                                        if ui.button("X").clicked() {
                                                            commands.push(
                                                                InspectCommand::RemoveComponent(
                                                                    e.id(),
                                                                    t_id,
                                                                ),
                                                            );
                                                        }
                                                    },
                                                );
                                            },
                                        );
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
                    let _counter = 0;
                    for idx in 0..components_id.len() {
                        let c_id = components_id[idx];
                        let _t_id = types_id[idx];
                        let name = pretty_type_name::pretty_type_name_str(
                            cell.components().get_info(c_id).unwrap().name(),
                        );

                        if name.to_lowercase().contains(&lower_filter) {
                            ui.label(name);
                            if ui.button("+").clicked() {
                                let id =
                                    cell.components().get_info(c_id).unwrap().type_id().unwrap();
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
