use std::sync::Arc;

use bevy::{prelude::*, utils::HashMap};
use bevy_egui::{
    egui::{collapsing_header::CollapsingState, CollapsingHeader},
    *,
};
use editor_core::prelude::*;
use prefab::editor_registry::EditorRegistry;
use undo::{AddedEntity, NewChange, RemovedEntity, UndoSet};

use crate::ui_registration::BundleReg;
use shared::*;

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

type HierarchyQueryIter<'a> = (
    Entity,
    Option<&'a Name>,
    Option<&'a Children>,
    Option<&'a Parent>,
);

/// System to show hierarchy
pub fn show_hierarchy(
    mut commands: Commands,
    query: Query<HierarchyQueryIter, With<PrefabMarker>>,
    mut selected: Query<Entity, With<Selected>>,
    mut clone_events: EventWriter<CloneEvent>,
    ui_reg: Res<BundleReg>,
    mut ui: NonSendMut<EditorUiRef>,
    mut changes: EventWriter<NewChange>,
) {
    let mut all: Vec<_> = query.iter().collect();
    all.sort_by_key(|a| a.0);

    let ui = &mut ui.0;
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (entity, _name, _children, parent) in all.iter() {
            if parent.is_none() {
                draw_entity(
                    &mut commands,
                    ui,
                    &query,
                    *entity,
                    &mut selected,
                    &mut clone_events,
                    &mut changes,
                );
            }
        }
        ui.vertical_centered(|ui| {
            ui.separator();
            if ui.button("+ Add new entity").clicked() {
                let id = commands.spawn_empty().insert(PrefabMarker).id();
                changes.send(NewChange {
                    change: Arc::new(AddedEntity { entity: id }),
                });
            }
            if ui.button("Clear all entities").clicked() {
                for (entity, _, _, _parent) in all.iter() {
                    commands.entity(*entity).despawn_recursive();

                    changes.send(NewChange {
                        change: Arc::new(RemovedEntity { entity: *entity }),
                    });
                }
            }
        });

        ui.label("Spawnable bundles");
        for (cat_name, cat) in ui_reg.bundles.iter() {
            ui.menu_button(cat_name, |ui| {
                for (name, dyn_bundle) in cat {
                    if ui.button(name).clicked() {
                        let entity = dyn_bundle.spawn(&mut commands);
                        changes.send(NewChange {
                            change: Arc::new(AddedEntity { entity }),
                        });
                    }
                }
            });
        }
    });
}

type DrawIter<'a> = (
    Entity,
    Option<&'a Name>,
    Option<&'a Children>,
    Option<&'a Parent>,
);

fn draw_entity(
    commands: &mut Commands,
    ui: &mut egui::Ui,
    query: &Query<DrawIter, With<PrefabMarker>>,
    entity: Entity,
    selected: &mut Query<Entity, With<Selected>>,
    clone_events: &mut EventWriter<CloneEvent>,
    changes: &mut EventWriter<NewChange>,
) {
    let Ok((_, name, children, parent)) = query.get(entity) else {
        return;
    };

    let entity_name = name.map_or_else(
        || format!("Entity ({:?})", entity),
        |name| format!("{} ({:?})", name.as_str(), entity),
    );

    let is_selected = selected.contains(entity);

    let label = if children.is_some() {
        CollapsingState::load_with_default_open(
            ui.ctx(),
            ui.make_persistent_id(entity_name.clone()),
            true,
        )
        .show_header(ui, |ui| {
            ui.selectable_label(is_selected, entity_name)
                .context_menu(|ui| {
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
                    if !selected.is_empty()
                        && !selected.contains(entity)
                        && ui.button("Attach to").clicked()
                    {
                        for e in selected.iter() {
                            commands.entity(entity).add_child(e);
                        }
                    }
                    if parent.is_some() && ui.button("Detach").clicked() {
                        commands.entity(entity).remove_parent();
                    }
                })
        })
        .body(|ui| {
            for child in children.unwrap().iter() {
                draw_entity(commands, ui, query, *child, selected, clone_events, changes);
            }
        })
        .1
        .inner
    } else {
        ui.selectable_label(is_selected, entity_name)
            .context_menu(|ui| {
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
                if !selected.is_empty()
                    && !selected.contains(entity)
                    && ui.button("Attach to").clicked()
                {
                    for e in selected.iter() {
                        commands.entity(entity).add_child(e);
                    }
                }
                if parent.is_some() && ui.button("Detach").clicked() {
                    commands.entity(entity).remove_parent();
                }
            })
    };

    if label.clicked() {
        if !is_selected {
            if !ui.input(|i| i.modifiers.shift) {
                for e in selected.iter() {
                    commands.entity(e).remove::<Selected>();
                }
            }
            commands.entity(entity).insert(Selected);
        } else {
            commands.entity(entity).remove::<Selected>();
        }
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
