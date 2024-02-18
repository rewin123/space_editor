pub mod components_order;
pub mod events_dispatcher;
pub mod refl_impl;
pub mod resources;
pub mod runtime_assets;

use std::any::TypeId;

use bevy::{
    ecs::{change_detection::MutUntyped, system::CommandQueue},
    prelude::*,
    ptr::PtrMut,
    reflect::ReflectFromPtr,
    utils::HashMap,
};

use bevy_egui_next::*;

use space_editor_core::prelude::*;
use space_prefab::{component::EntityLink, editor_registry::EditorRegistry};
use space_shared::ext::bevy_inspector_egui::{
    self, inspector_egui_impls::InspectorEguiImpl, reflect_inspector::InspectorUi,
};

use crate::{colors::DEFAULT_BG_COLOR, icons::add_component_icon, sizing::Sizing};

use self::{
    components_order::{ComponentsOrder, ComponentsPriority},
    events_dispatcher::EventDispatcherTab,
    refl_impl::{entity_ref_ui, entity_ref_ui_readonly, many_unimplemented},
    resources::ResourceTab,
    runtime_assets::RuntimeAssetsTab,
};

use super::{
    editor_tab::{EditorTab, EditorTabName},
    EditorUiAppExt,
};

/// Entities with this marker will be skiped in inspector
#[derive(Component)]
pub struct SkipInspector;

/// Plugin to activate components inspector
pub struct SpaceInspectorPlugin;

impl Plugin for SpaceInspectorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InspectState>();
        app.init_resource::<FilterComponentState>();
        app.init_resource::<ComponentsOrder>();
        app.editor_component_priority::<Name>(0);
        app.editor_component_priority::<Transform>(1);

        app.editor_tab_by_trait(EditorTabName::Inspector, InspectorTab::default());
        app.editor_tab_by_trait(EditorTabName::Resource, ResourceTab::default());
        app.editor_tab_by_trait(
            EditorTabName::EventDispatcher,
            EventDispatcherTab::default(),
        );
        app.editor_tab_by_trait(EditorTabName::RuntimeAssets, RuntimeAssetsTab::default());

        app.add_systems(Update, execute_inspect_command);

        app.add_systems(Startup, register_custom_impls);
    }
}

#[derive(Resource, Default)]
pub struct InspectorTab {
    open_components: HashMap<String, bool>,
}

impl EditorTab for InspectorTab {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut Commands, world: &mut World) {
        inspect(ui, world, &mut self.open_components);
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
pub fn inspect(ui: &mut egui::Ui, world: &mut World, open_components: &mut HashMap<String, bool>) {
    let sizing = world.resource::<Sizing>().clone();
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
    let mut disable_pan_orbit = false;

    //Collet data about all components
    let components_priority = world.resource::<ComponentsOrder>().clone().components;
    let mut components_id = Vec::new();
    for reg in registry.iter() {
        if let Some(c_id) = world.components().get_id(reg.type_id()) {
            let name = pretty_type_name::pretty_type_name_str(
                world.components().get_info(c_id).unwrap().name(),
            );
            let priority = components_priority.get(&name).unwrap_or(&u8::MAX);
            components_id.push((c_id, reg.type_id(), name, priority));
        }
    }
    components_id.sort_by(|(.., name_a, priority_a), (.., name_b, priority_b)| {
        priority_a.cmp(priority_b).then(name_a.cmp(name_b))
    });

    // println!("{:#?}\n", components_id);

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

    //Open context window by right mouse button click
    //ui.interact blocks all control of inspector window
    if ui
        .interact(ui.min_rect(), "painter".into(), egui::Sense::click())
        .secondary_clicked()
    {
        state.show_add_component_window = true;
    }

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
                for (c_id, t_id, name, _) in &components_id {
                    if let Some(data) = unsafe { e.get_mut_by_id(*c_id) } {
                        let registration = registry.get(*t_id).unwrap();
                        if let Some(reflect_from_ptr) = registration.data::<ReflectFromPtr>() {
                            let (ptr, mut set_changed) = mut_untyped_split(data);

                            let value = unsafe { reflect_from_ptr.from_ptr_mut()(ptr) };

                            if !editor_registry.silent.contains(&registration.type_id()) {
                                ui.push_id(format!("{:?}-{}", &e.id(), &name), |ui| {
                                    let header = egui::CollapsingHeader::new(name)
                                        .default_open(*open_components.get(name).unwrap_or(&false))
                                        .show(ui, |ui| {
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
                                    if header.header_response.clicked() {
                                        let open_name =
                                            open_components.entry(name.clone()).or_default();
                                        //At click header not opened simultaneously so its need to check percent of opened
                                        *open_name = header.openness < 0.5;
                                    }
                                });

                                ui.push_id(
                                    format!("del component {:?}-{}", &e.id(), &name),
                                    |ui| {
                                        //must be on top
                                        ui.with_layout(
                                            egui::Layout::top_down(egui::Align::Min),
                                            |ui| {
                                                let button =
                                                    egui::Button::new("🗙").fill(DEFAULT_BG_COLOR);
                                                if ui.add(button).clicked() {
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

    let width = ui.available_width();
    let add_component_str = "Add component";
    let pixel_count = add_component_str.len() as f32 * 8.;
    let x_padding = (width - pixel_count - 16. - sizing.icon.to_size()) / 2.;

    //Open context window by button
    ui.vertical_centered(|ui| {
        ui.spacing();
        ui.style_mut().spacing.button_padding = egui::Vec2 {
            x: x_padding,
            y: 2.,
        };
        if ui
            .add(add_component_icon(
                sizing.icon.to_size(),
                sizing.icon.to_size(),
                add_component_str,
            ))
            .clicked()
        {
            state.show_add_component_window = true;
        }
    });

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
                for (c_id, _t_id, name, _) in &components_id {
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

    //All works with this if statement. Dont change it and dont add is_using_pointer() method
    if ui.ui_contains_pointer() {
        disable_pan_orbit = true;
    }

    state.commands = commands;

    if disable_pan_orbit {
        world.resource_mut::<crate::EditorCameraEnabled>().0 = false;
    }
}
