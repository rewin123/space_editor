pub mod components_order;
pub mod events_dispatcher;
pub mod refl_impl;
pub mod resources;
pub mod runtime_assets;

use std::any::TypeId;

use bevy::{
    ecs::{change_detection::MutUntyped, world::CommandQueue},
    prelude::*,
    ptr::PtrMut,
    reflect::ReflectFromPtr,
    utils::HashMap,
};

use bevy_egui::{egui::TextEdit, *};

use space_editor_core::prelude::*;
use space_prefab::{component::EntityLink, editor_registry::EditorRegistry};
use space_shared::{
    ext::bevy_inspector_egui::{
        inspector_egui_impls::InspectorEguiImpl, reflect_inspector::InspectorUi,
    },
    toast::{ToastKind, ToastMessage},
};

use crate::{editor_tab_name::EditorTabName, icons::add_component_icon};
use space_editor_tabs::prelude::*;

use self::{
    components_order::{ComponentsOrder, ComponentsPriority},
    events_dispatcher::EventDispatcherTab,
    refl_impl::{entity_ref_ui, entity_ref_ui_readonly, many_unimplemented},
    resources::ResourceTab,
    runtime_assets::RuntimeAssetsTab,
};
use crate::{colors::*, sizing::Sizing};

/// Entities with this marker will be skipped in inspector
#[derive(Component)]
pub struct SkipInspector;

/// Plugin to activate components inspector
pub struct SpaceInspectorPlugin;

impl Plugin for SpaceInspectorPlugin {
    #[cfg(not(tarpaulin_include))]
    fn build(&self, app: &mut App) {
        app.init_resource::<InspectState>();
        app.init_resource::<FilterComponentState>();
        app.init_resource::<ComponentsOrder>();
        app.editor_component_priority::<Name>(0);
        app.editor_component_priority::<Transform>(1);

        app.editor_tab_by_trait(InspectorTab::default());
        app.editor_tab_by_trait(ResourceTab::default());
        app.editor_tab_by_trait(EventDispatcherTab::default());
        app.editor_tab_by_trait(RuntimeAssetsTab::default());

        app.add_systems(Update, execute_inspect_command);

        app.add_systems(Startup, register_custom_impls);
    }
}

#[derive(Resource, Default)]
pub struct InspectorTab {
    open_components: HashMap<String, bool>,
    show_all_components: bool,
}

impl EditorTab for InspectorTab {
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut Commands, world: &mut World) {
        // Defaults in case it is missing
        let sizing = world.get_resource::<Sizing>().cloned().unwrap_or_default();
        let selected_entity = world
            .query_filtered::<Entity, With<Selected>>()
            .get_single(world);

        let Ok(selected_entity) = selected_entity else {
            return;
        };

        let editor_registry_resource = world.resource::<EditorRegistry>().clone();
        let editor_regitry_handle = editor_registry_resource.registry.clone();
        let editor_registry = editor_regitry_handle.read();

        let app_registry_handle = world.resource::<AppTypeRegistry>().clone();
        let app_registry = app_registry_handle.read();
        let mut disable_pan_orbit = false;

        //Collet data about all components | empty if missing
        let components_priority = world
            .get_resource::<ComponentsOrder>()
            .cloned()
            .unwrap_or_default()
            .components;
        let mut components_id = Vec::new();
        if self.show_all_components {
            for reg in app_registry.iter() {
                if let Some(c_id) = world.components().get_id(reg.type_id()) {
                    let name = pretty_type_name::pretty_type_name_str(
                        world.components().get_info(c_id).unwrap().name(),
                    );
                    let priority = components_priority.get(&name).unwrap_or(&u8::MAX);
                    components_id.push((c_id, reg.type_id(), name, priority));
                }
            }
        } else {
            for reg in editor_registry.iter() {
                if let Some(c_id) = world.components().get_id(reg.type_id()) {
                    let name = pretty_type_name::pretty_type_name_str(
                        world.components().get_info(c_id).unwrap().name(),
                    );
                    let priority = components_priority.get(&name).unwrap_or(&u8::MAX);
                    components_id.push((c_id, reg.type_id(), name, priority));
                }
            }
        }
        components_id.sort_by(|(.., name_a, priority_a), (.., name_b, priority_b)| {
            priority_a.cmp(priority_b).then(name_a.cmp(name_b))
        });

        let cell = world.as_unsafe_world_cell();
        let Some(mut state) = (unsafe { cell.get_resource_mut::<InspectState>() }) else {
            error!("Failed to load inspect state");
            world.send_event(ToastMessage::new(
                "Failed to load inspect state",
                ToastKind::Error,
            ));
            return;
        };

        let mut commands: Vec<InspectCommand> = vec![];
        let mut queue = CommandQueue::default();
        let mut cx = unsafe {
            bevy_inspector_egui::reflect_inspector::Context {
                world: Some(cell.world_mut().into()),
                queue: Some(&mut queue),
            }
        };
        let mut env = InspectorUi::for_bevy(&app_registry, &mut cx);

        //Open context window by right mouse button click
        //ui.interact blocks all control of inspector window
        if ui
            .interact(ui.min_rect(), "painter".into(), egui::Sense::click())
            .secondary_clicked()
        {
            state.show_add_component_window = true;
        }

        let add_component_width = ui.available_width();
        let add_component_str = to_label("Add component", sizing.text);
        let add_component_pixel_count =
            add_component_str.text().len() as f32 * 8. * sizing.text / 12.;
        let add_component_x_padding =
            (add_component_width - add_component_pixel_count - 16. - sizing.icon.to_size()) / 2.;

        let components_area = egui::ScrollArea::vertical().show(ui, |ui| {
            if let Some(e) = cell.get_entity(selected_entity) {
                let mut name;
                if let Some(name_struct) = unsafe { e.get::<Name>() } {
                    name = name_struct.as_str().to_string();
                    if name.is_empty() {
                        name = format!("{} (empty name)", e.id());
                    }
                } else {
                    name = format!("{}", e.id());
                }
                ui.heading(&name);
                let mut state = unsafe { cell.get_resource_mut::<FilterComponentState>().unwrap() };
                ui.horizontal(|ui| {
                    let button_size = ui
                        .style()
                        .text_styles
                        .get(&egui::TextStyle::Button)
                        .map(|f| f.size)
                        .unwrap_or(14.);
                    let button_padding = ui.style().spacing.button_padding.x * 2.;
                    let space = ui.style().spacing.item_spacing.x;
                    let width =
                        2.0f32.mul_add(-space, ui.available_width() - button_size - button_padding);
                    ui.add(
                        TextEdit::singleline(&mut state.component_add_filter).desired_width(width),
                    );
                    if ui.button("🗑").on_hover_text("Clear filter").clicked() {
                        state.component_add_filter.clear();
                    }
                });
                let lower_filter = state.component_add_filter.to_lowercase();
                let e_id = e.id().index();

                ui.label("Components:");
                egui::Grid::new(format!("{e_id}")).show(ui, |ui| {
                    for (c_id, t_id, name, _) in &components_id {
                        if name.to_lowercase().contains(&lower_filter) {
                            if let Some(data) = unsafe { e.get_mut_by_id(*c_id) } {
                                let mut is_editor_component = false;
                                let registration;
                                if let Some(reg) = editor_registry.get(*t_id) {
                                    is_editor_component = true;
                                    registration = reg;
                                } else {
                                    registration = app_registry.get(*t_id).unwrap();
                                }

                                if let Some(reflect_from_ptr) =
                                    registration.data::<ReflectFromPtr>()
                                {
                                    let (ptr, mut set_changed) = mut_untyped_split(data);

                                    let value = unsafe { reflect_from_ptr.from_ptr_mut()(ptr) };

                                    if is_editor_component {
                                        if !editor_registry_resource
                                            .silent
                                            .contains(&registration.type_id())
                                        {
                                            self.show_component(
                                                ui,
                                                e,
                                                name,
                                                &mut env,
                                                value,
                                                &mut set_changed,
                                            );

                                            ui.push_id(
                                                format!("del component {:?}-{}", &e.id(), &name),
                                                |ui| {
                                                    //must be on top
                                                    ui.with_layout(
                                                        egui::Layout::top_down(egui::Align::Min),
                                                        |ui| {
                                                            let button = egui::Button::new("🗙")
                                                                .fill(DEFAULT_BG_COLOR);
                                                            if ui.add(button).clicked() {
                                                                commands.push(
                                                                    InspectCommand::RemoveComponent(
                                                                        e.id(),
                                                                        *t_id,
                                                                    ),
                                                                );
                                                            }
                                                        },
                                                    );
                                                },
                                            );
                                            ui.end_row();
                                        }
                                    } else {
                                        self.show_component(
                                            ui,
                                            e,
                                            name,
                                            &mut env,
                                            value,
                                            &mut set_changed,
                                        );
                                        ui.end_row();
                                    }
                                }
                            }
                        }
                    }
                });

                ui.separator();
            }
        });
        ui.checkbox(&mut self.show_all_components, "Show non editor components");
        ui.spacing();

        // Shifts `add compoenent` button full left in case width is not large enough
        // for all components widths
        if add_component_width > 1.35 * (add_component_pixel_count + 16. + sizing.icon.to_size()) {
            //Open context window by button
            ui.vertical_centered(|ui| {
                ui.spacing();
                ui.style_mut().spacing.button_padding = egui::Vec2 {
                    x: add_component_x_padding,
                    y: 2.,
                };

                if ui
                    .add(add_component_icon(sizing.icon.to_size(), add_component_str))
                    .clicked()
                {
                    state.show_add_component_window = true;
                }
            });
        } else {
            ui.spacing();
            ui.style_mut().spacing.button_padding = egui::Vec2 { x: 16., y: 2. };

            if ui
                .add(add_component_icon(sizing.icon.to_size(), add_component_str))
                .clicked()
            {
                state.show_add_component_window = true;
            }
        }

        egui::Window::new("Add component")
            .open(&mut state.show_add_component_window)
            .resizable(true)
            .scroll([false, true])
            .default_width(120.)
            .default_height(300.)
            .default_pos(components_area.inner_rect.center_bottom())
            .show(ui.ctx(), |ui: &mut egui::Ui| {
                let mut state = unsafe { cell.get_resource_mut::<FilterComponentState>().unwrap() };
                ui.horizontal(|ui| {
                    let button_size = ui
                        .style()
                        .text_styles
                        .get(&egui::TextStyle::Button)
                        .map(|f| f.size)
                        .unwrap_or(14.);
                    let button_padding = ui.style().spacing.button_padding.x * 2.;
                    let space = ui.style().spacing.item_spacing.x;
                    let width =
                        2.0f32.mul_add(-space, ui.available_width() - button_size - button_padding);
                    ui.add(
                        TextEdit::singleline(&mut state.component_add_filter).desired_width(width),
                    );
                    if ui.button("🗑").on_hover_text("Clear filter").clicked() {
                        state.component_add_filter.clear();
                    }
                });
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

        if let (Some(mut editor_camera_enabled), true) = (
            world.get_resource_mut::<crate::EditorCameraEnabled>(),
            disable_pan_orbit,
        ) {
            editor_camera_enabled.0 = false;
        } else {
            // error!("Failed to get editor camera config");
        }
    }

    fn tab_name(&self) -> space_editor_tabs::tab_name::TabNameHolder {
        space_editor_tabs::tab_name::TabNameHolder::new(EditorTabName::Inspector)
    }
}

impl InspectorTab {
    fn show_component(
        &mut self,
        ui: &mut egui::Ui,
        e: bevy::ecs::world::unsafe_world_cell::UnsafeEntityCell<'_>,
        name: &String,
        env: &mut InspectorUi<'_, '_>,
        value: &mut dyn Reflect,
        set_changed: &mut impl FnMut(),
    ) {
        ui.push_id(format!("{:?}-{}", &e.id(), &name), |ui| {
            let default = name.to_lowercase() == *"transform";
            let header = egui::CollapsingHeader::new(name)
                .default_open(*self.open_components.get(name).unwrap_or(&default))
                .show(ui, |ui| {
                    ui.push_id(format!("content-{:?}-{}", &e.id(), &name), |ui| {
                        if env.ui_for_reflect_with_options(value, ui, ui.id(), &()) {
                            (set_changed)();
                        }
                    });
                });
            if header.header_response.clicked() {
                let open_name = self.open_components.entry(name.clone()).or_default();
                //At click header not opened simultaneously so its need to check percent of opened
                *open_name = header.openness < 0.5;
            }
        });
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
