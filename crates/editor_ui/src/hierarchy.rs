#![allow(clippy::too_many_arguments)]
use std::sync::Arc;

use bevy::{ecs::query::ReadOnlyWorldQuery, prelude::*, utils::HashMap};
use bevy_egui_next::{egui::collapsing_header::CollapsingState, *};
use space_editor_core::prelude::*;
use space_prefab::{component::SceneAutoChild, editor_registry::EditorRegistry};
use space_undo::{AddedEntity, NewChange, RemovedEntity, UndoSet};

use space_shared::*;

use super::{editor_tab::EditorTabName, EditorUiAppExt, EditorUiRef};

/// Event to clone entity with clone all registered components
#[derive(Event)]
pub struct CloneEvent {
    pub id: Entity,
}

/// Plugin to activate hierarchy UI in editor UI
#[derive(Default)]
pub struct SpaceHierarchyPlugin {}

impl Plugin for SpaceHierarchyPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<SelectedPlugin>() {
            app.add_plugins(SelectedPlugin);
        }

        app.init_resource::<HierarchyTabState>();
        app.editor_tab(EditorTabName::Hierarchy, "Hierarchy".into(), show_hierarchy);

        // app.add_systems(Update, show_hierarchy.before(crate::editor::ui_camera_block).in_set(EditorSet::Editor));
        app.add_systems(Update, clone_enitites.in_set(EditorSet::Editor));
        app.add_systems(
            PostUpdate,
            detect_cloned_entities
                .in_set(EditorSet::Editor)
                .before(UndoSet::PerType),
        );
        app.add_event::<CloneEvent>();
    }
}

#[derive(Resource, Default)]
pub struct HierarchyTabState {
    pub show_editor_entities: bool,
    pub show_spawnable_bundles: bool,
    pub entity_filter: String,
}

pub type HierarchyQueryIter<'a> = (
    Entity,
    Option<&'a Name>,
    Option<&'a Children>,
    Option<&'a Parent>,
);

/// System to show hierarchy
pub fn show_hierarchy(
    mut commands: Commands,
    query: Query<HierarchyQueryIter, With<PrefabMarker>>,
    all_entities: Query<HierarchyQueryIter>,
    mut selected: Query<Entity, With<Selected>>,
    mut clone_events: EventWriter<CloneEvent>,
    mut ui: NonSendMut<EditorUiRef>,
    mut changes: EventWriter<NewChange>,
    mut state: ResMut<HierarchyTabState>,
    auto_children: Query<(), With<SceneAutoChild>>,
) {
    let mut all: Vec<_> = if state.show_editor_entities {
        all_entities.iter().collect()
    } else {
        query.iter().collect()
    };
    all.sort_by_key(|a| a.0);
    let ui = &mut ui.0;
    ui.text_edit_singleline(&mut state.entity_filter);
    ui.spacing();
    let lower_filter = state.entity_filter.to_lowercase();

    egui::ScrollArea::vertical().show(ui, |ui| {
        for (entity, _name, _children, parent) in all.iter().filter(|(_, name, _, _)| {
            name.map(|n| n.to_lowercase())
                .unwrap_or_else(|| "entity".to_string())
                .contains(&lower_filter)
        }) {
            if parent.is_none() {
                if state.show_editor_entities {
                    draw_entity::<()>(
                        &mut commands,
                        ui,
                        &all_entities,
                        *entity,
                        &mut selected,
                        &mut clone_events,
                        &mut changes,
                        &auto_children,
                    );
                } else {
                    draw_entity::<With<PrefabMarker>>(
                        &mut commands,
                        ui,
                        &query,
                        *entity,
                        &mut selected,
                        &mut clone_events,
                        &mut changes,
                        &auto_children,
                    );
                }
            }
        }
    });
}

type DrawIter<'a> = (
    Entity,
    Option<&'a Name>,
    Option<&'a Children>,
    Option<&'a Parent>,
);

fn draw_entity<F: ReadOnlyWorldQuery>(
    commands: &mut Commands,
    ui: &mut egui::Ui,
    query: &Query<DrawIter, F>,
    entity: Entity,
    selected: &mut Query<Entity, With<Selected>>,
    clone_events: &mut EventWriter<CloneEvent>,
    changes: &mut EventWriter<NewChange>,
    auto_children: &Query<(), With<SceneAutoChild>>,
) {
    let Ok((_, name, children, parent)) = query.get(entity) else {
        return;
    };

    let entity_name = name.map_or_else(
        || format!("Entity ({:?})", entity),
        |name| format!("{} ({:?})", name.as_str(), entity),
    );

    let is_selected = selected.contains(entity);

    if children.is_some_and(|children| children.iter().any(|child| query.get(*child).is_ok())) {
        CollapsingState::load_with_default_open(
            ui.ctx(),
            ui.make_persistent_id(entity_name.clone()),
            true,
        )
        .show_header(ui, |ui| {
            let mut entity_name = egui::RichText::new(entity_name.clone());
            if auto_children.get(entity).is_ok() {
                entity_name = entity_name.italics();
            }

            let response = ui.selectable_label(is_selected, entity_name);
            let is_clicked = response.clicked();
            response.context_menu(|ui| {
                hierarchy_entity_context(
                    ui,
                    commands,
                    entity,
                    changes,
                    clone_events,
                    selected,
                    parent,
                );
            });

            if is_clicked {
                if is_selected {
                    commands.entity(entity).remove::<Selected>();
                    info!("Removed selected: {:?}", entity);
                } else {
                    commands.entity(entity).insert(Selected);

                    //check shift pressed
                    if !ui.input(|i| i.modifiers.shift) {
                        selected.for_each(|e| {
                            commands.entity(e).remove::<Selected>();
                        })
                    }
                    info!("Added selected: {:?}", entity);
                }
            }
        })
        .body(|ui| {
            for child in children.unwrap().iter() {
                draw_entity(
                    commands,
                    ui,
                    query,
                    *child,
                    selected,
                    clone_events,
                    changes,
                    auto_children,
                );
            }
        });
    } else {
        let selectable = ui.selectable_label(is_selected, format!("      {}", entity_name));
        let is_clicked = selectable.clicked();

        selectable.context_menu(|ui| {
            hierarchy_entity_context(
                ui,
                commands,
                entity,
                changes,
                clone_events,
                selected,
                parent,
            );
        });

        if is_clicked {
            if is_selected {
                commands.entity(entity).remove::<Selected>();
                info!("Removed selected: {:?}", entity);
            } else {
                commands.entity(entity).insert(Selected);

                //check shift pressed
                if !ui.input(|i| i.modifiers.shift) {
                    selected.for_each(|e| {
                        commands.entity(e).remove::<Selected>();
                    })
                }
                info!("Added selected: {:?}", entity);
            }
        }
    };
}

fn hierarchy_entity_context(
    ui: &mut egui::Ui,
    commands: &mut Commands<'_, '_>,
    entity: Entity,
    changes: &mut EventWriter<'_, NewChange>,
    clone_events: &mut EventWriter<'_, CloneEvent>,
    selected: &mut Query<'_, '_, Entity, With<Selected>>,
    parent: Option<&Parent>,
) {
    if ui.button("Add child").clicked() {
        let new_id = commands.spawn_empty().insert(PrefabMarker).id();
        commands.entity(entity).add_child(new_id);
        changes.send(NewChange {
            change: Arc::new(AddedEntity { entity: new_id }),
        });
        ui.close_menu();
    }
    if ui.button("Delete").clicked() {
        commands.entity(entity).despawn_recursive();
        changes.send(NewChange {
            change: Arc::new(RemovedEntity { entity }),
        });
        ui.close_menu();
    }
    if ui.button("Clone").clicked() {
        clone_events.send(CloneEvent { id: entity });
        ui.close_menu();
    }
    if !selected.is_empty() && !selected.contains(entity) && ui.button("Attach to").clicked() {
        for e in selected.iter() {
            commands.entity(entity).add_child(e);
        }
    }
    if parent.is_some() && ui.button("Detach").clicked() {
        commands.entity(entity).remove_parent();
    }
}

#[derive(Component)]
pub struct ClonedEntity;

fn clone_enitites(
    mut commands: Commands,
    query: Query<EntityRef>,
    mut events: EventReader<CloneEvent>,
    editor_registry: Res<EditorRegistry>,
) {
    for event in events.read() {
        let mut queue = vec![(event.id, commands.spawn_empty().id())];
        let mut map = HashMap::new();

        while let Some((src_id, dst_id)) = queue.pop() {
            map.insert(src_id, dst_id);
            if let Ok(entity) = query.get(src_id) {
                if entity.contains::<PrefabMarker>() {
                    let mut cmds = commands.entity(dst_id);
                    cmds.insert(ClonedEntity);

                    editor_registry.clone_entity_flat(&mut cmds, &entity);

                    if let Some(parent) = entity.get::<Parent>() {
                        if let Some(new_parent) = map.get(&parent.get()) {
                            commands.entity(*new_parent).add_child(dst_id);
                        } else {
                            commands.entity(parent.get()).add_child(dst_id);
                        }
                    }

                    if let Some(children) = entity.get::<Children>() {
                        for id in children {
                            queue.push((*id, commands.spawn_empty().id()));
                        }
                    }
                }
            }
        }
    }
    events.clear();
}

fn detect_cloned_entities(
    mut commands: Commands,
    query: Query<Entity, Added<ClonedEntity>>,
    mut changes: EventWriter<NewChange>,
) {
    for entity in query.iter() {
        commands.entity(entity).remove::<ClonedEntity>();
        changes.send(NewChange {
            change: Arc::new(AddedEntity { entity }),
        });
    }
}
