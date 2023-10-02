use bevy::{prelude::*, utils::HashMap};
use bevy_egui::*;

use crate::{
    editor::ui_registration::BundleReg,
    prelude::{EditorRegistry, Selected, SelectedPlugin},
    EditorSet, PrefabMarker,
};

use super::{EditorUiAppExt, EditorUiRef};

/// Event to clone entity with clone all registered components
#[derive(Event)]
pub struct CloneEvent {
    id: Entity,
}

/// Plugin to activate hierarchy UI in editor UI
#[derive(Default)]
pub struct SpaceHierarchyPlugin {}

impl Plugin for SpaceHierarchyPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<SelectedPlugin>() {
            app.add_plugins(SelectedPlugin);
        }

        app.editor_tab(
            crate::prelude::EditorTabName::Hierarchy,
            "Hierarchy".into(),
            show_hierarchy,
        );

        // app.add_systems(Update, show_hierarchy.before(crate::editor::ui_camera_block).in_set(EditorSet::Editor));
        app.add_systems(Update, clone_enitites.in_set(EditorSet::Editor));
        app.add_event::<CloneEvent>();
    }
}

/// System to show hierarchy
pub fn show_hierarchy(
    mut commands: Commands,
    query: Query<(Entity, Option<&Name>, Option<&Children>, Option<&Parent>), With<PrefabMarker>>,
    mut selected: Query<Entity, With<Selected>>,
    mut clone_events: EventWriter<CloneEvent>,
    ui_reg: Res<BundleReg>,
    mut ui: NonSendMut<EditorUiRef>,
) {
    let mut all: Vec<_> = query.iter().collect();
    all.sort_by_key(|a| a.0);
    let pointer_used = false;

    let ui = &mut ui.0;
    egui::ScrollArea::vertical().show(ui, |ui| {
        for (entity, _, _, parent) in all.iter() {
            if parent.is_none() {
                draw_entity(
                    &mut commands,
                    ui,
                    &query,
                    *entity,
                    &mut selected,
                    &mut clone_events,
                    pointer_used,
                );
            }
        }
        ui.vertical_centered(|ui| {
            if ui.button("----- + -----").clicked() {
                commands.spawn_empty().insert(PrefabMarker);
            }
            if ui.button("Clear all").clicked() {
                for (entity, _, _, _parent) in all.iter() {
                    commands.entity(*entity).despawn_recursive();
                }
            }
        });

        ui.label("Spawn bundle");
        for (cat_name, cat) in ui_reg.bundles.iter() {
            ui.menu_button(cat_name, |ui| {
                for (name, dyn_bundle) in cat {
                    if ui.button(name).clicked() {
                        let _entity = dyn_bundle.spawn(&mut commands);
                    }
                }
            });
        }
    });
}

fn draw_entity(
    commands: &mut Commands,
    ui: &mut egui::Ui,
    query: &Query<(Entity, Option<&Name>, Option<&Children>, Option<&Parent>), With<PrefabMarker>>,
    entity: Entity,
    selected: &mut Query<Entity, With<Selected>>,
    clone_events: &mut EventWriter<CloneEvent>,
    pointer_used: bool,
) {
    let Ok((_, name, children, parent)) = query.get(entity) else {
        return;
    };

    let entity_name = name.map_or_else(
        || format!("Entity {:?}", entity),
        |name| format!("Entity {:?}: {:?}", entity, name.as_str()),
    );

    ui.indent(entity_name.clone(), |ui| {
        let is_selected = selected.contains(entity);

        let label = ui
            .selectable_label(is_selected, entity_name)
            .context_menu(|ui| {
                if ui.button("Add child").clicked() {
                    let new_id = commands.spawn_empty().insert(PrefabMarker).id();
                    commands.entity(entity).add_child(new_id);
                    ui.close_menu();
                }
                if ui.button("Delete").clicked() {
                    commands.entity(entity).despawn_recursive();
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
            });

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

        if let Some(children) = children {
            for child in children.iter() {
                draw_entity(
                    commands,
                    ui,
                    query,
                    *child,
                    selected,
                    clone_events,
                    pointer_used,
                );
            }
        }
    });
}

fn clone_enitites(
    mut commands: Commands,
    query: Query<EntityRef>,
    mut events: EventReader<CloneEvent>,
    editor_registry: Res<EditorRegistry>,
) {
    for event in events.into_iter() {
        let mut queue = vec![(event.id, commands.spawn_empty().id())];
        let mut map = HashMap::new();

        while let Some((src_id, dst_id)) = queue.pop() {
            map.insert(src_id, dst_id);
            if let Ok(entity) = query.get(src_id) {
                if entity.contains::<PrefabMarker>() {
                    let mut cmds = commands.entity(dst_id);

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
