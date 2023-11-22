pub mod refl_impl;

use std::any::TypeId;

use bevy::{
    ecs::{change_detection::MutUntyped, system::CommandQueue},
    prelude::*,
    ptr::PtrMut,
    reflect::ReflectFromPtr,
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
        app.init_resource::<FilterComponentState>();

        app.editor_tab_by_trait(
            crate::prelude::EditorTabName::Inspector,
            InspectorTab::default(),
        );

        app.add_systems(Update, execute_inspect_command);

        app.add_systems(Startup, register_custom_impls);
    }
}

#[derive(Resource, Default)]
pub struct InspectorTab {}

impl EditorTab for InspectorTab {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut Commands, world: &mut World) {
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
#[derive(Resource, Default)]
struct InspectState {
    commands: Vec<InspectCommand>,
    show_add_component_window: bool,
}

#[derive(Resource, Default)]
struct FilterComponentState {
    component_add_filter: String,
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
    let selected_entity = world
        .query_filtered::<Entity, With<Selected>>()
        .get_single(world);

    let Ok(selected_entity) = selected_entity else {
        return;
    };

    let editor_registry = world.resource::<EditorRegistry>().clone();
    let all_registry = editor_registry.registry.clone();
    let registry = all_registry.read();
    let app_registry = world.resource::<AppTypeRegistry>().clone();
    let world_registry = app_registry.read();
    let disable_pan_orbit = false;

    //Collet data about all components
    let mut components_id = Vec::new();
    for reg in registry.iter() {
        if let Some(c_id) = world.components().get_id(reg.type_id()) {
            let name = pretty_type_name::pretty_type_name_str(
                world.components().get_info(c_id).unwrap().name(),
            );
            components_id.push((c_id, reg.type_id(), name));
        }
    }
    components_id.sort_by(|a, b| a.2.cmp(&b.2));

    let cell = world.as_unsafe_world_cell();
    let mut state = unsafe { cell.get_resource_mut::<InspectState>().unwrap() };

    let mut commands: Vec<InspectCommand> = vec![];
    let mut queue = CommandQueue::default();
    let mut cx = unsafe {
        bevy_inspector_egui::reflect_inspector::Context {
            world: Some(cell.world_mut().into()),
            queue: Some(&mut queue),
        }
    };
    let mut env = InspectorUi::for_bevy(&world_registry, &mut cx);

    let components_area = egui::ScrollArea::vertical().show(ui, |ui| {
        if let Some(e) = cell.get_entity(selected_entity) {
            let mut name;
            if let Some(name_struct) = unsafe { e.get::<Name>() } {
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
                for (c_id, t_id, name) in &components_id {
                    if let Some(data) = unsafe { e.get_mut_by_id(*c_id) } {
                        let registration = registry.get(*t_id).unwrap();
                        if let Some(reflect_from_ptr) = registration.data::<ReflectFromPtr>() {
                            let (ptr, mut set_changed) = mut_untyped_split(data);

                            let value = unsafe { reflect_from_ptr.from_ptr_mut()(ptr) };

                            if !editor_registry.silent.contains(&registration.type_id()) {
                                ui.push_id(format!("{:?}-{}", &e.id(), &name), |ui| {
                                    ui.collapsing(name, |ui| {
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
                                                    commands.push(InspectCommand::RemoveComponent(
                                                        e.id(),
                                                        *t_id,
                                                    ));
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
    });

    let response = ui.interact(
        components_area.inner_rect,
        components_area.id,
        egui::Sense::click(),
    );
    if response.secondary_clicked() {
        state.show_add_component_window = true;
    }

    egui::Window::new("Add component")
        .open(&mut state.show_add_component_window)
        .resizable(true)
        .scroll2([false, true])
        .default_width(120.)
        .default_height(300.)
        .default_pos(components_area.inner_rect.center_bottom())
        .show(ui.ctx(), |ui: &mut egui::Ui| {
            let mut state = unsafe { cell.get_resource_mut::<FilterComponentState>().unwrap() };
            ui.text_edit_singleline(&mut state.component_add_filter);
            let lower_filter = state.component_add_filter.to_lowercase();
            egui::Grid::new("Component grid").show(ui, |ui| {
                let _counter = 0;
                for (c_id, _t_id, name) in &components_id {
                    if name.to_lowercase().contains(&lower_filter) {
                        ui.label(name);
                        if ui.button("+").clicked() {
                            let id = cell
                                .components()
                                .get_info(*c_id)
                                .unwrap()
                                .type_id()
                                .unwrap();
                            commands.push(InspectCommand::AddComponent(selected_entity, id));
                        }
                        ui.end_row();
                    }
                }
            });
        });

    state.commands = commands;

    if disable_pan_orbit {
        world.resource_mut::<crate::editor::PanOrbitEnabled>().0 = false;
    }
}
