use bevy::prelude::*;
use bevy_egui::*;

use super::selected::*;

pub struct SpaceHierarchyPlugin {

}

impl Default for SpaceHierarchyPlugin {
    fn default() -> Self {
        Self {

        }
    }
}

impl Plugin for SpaceHierarchyPlugin {
    fn build(&self, app: &mut App) {

        if !app.is_plugin_added::<SelectedPlugin>() {
            app.add_plugins(SelectedPlugin);
        }

        app.add_systems(Update, show_hierarchy);
    }
}


fn show_hierarchy(
    mut commands : Commands,
    mut contexts : EguiContexts,
    query: Query<(Entity, Option<&Name>, Option<&Children>, Option<&Parent>)>,
    mut selected : ResMut<SelectedEntities>
) {
    let mut all : Vec<_> = query.iter().collect();
    all.sort_by_key(|a| a.0);
    egui::SidePanel::left("Scene hierarchy")
            .show(contexts.ctx_mut(), |ui| {
        for (entity, _, _, parent) in all.iter() {
            if parent.is_none() {
                draw_entity(&mut commands, ui, &query, *entity, &mut selected);
            }
        }
        ui.vertical_centered(|ui| {
            if ui.button("----- + -----").clicked() {
                commands.spawn_empty();
            }
        });
    });
}

fn draw_entity(
    commands : &mut Commands, 
    ui: &mut egui::Ui,
    query: &Query<(Entity, Option<&Name>, Option<&Children>, Option<&Parent>)>,
    entity: Entity,
    selected : &mut SelectedEntities
) {
    let Ok((_, name, children, parent)) = query.get(entity) else {
        return;
    };

    let entity_name = name.map_or_else(
        || format!("Entity {:?}", entity),
        |name| format!("Entity {:?}: {:?}", entity, name.as_str()),
    );

    ui.indent(entity_name.clone(), |ui| {
        let is_selected = selected.list.contains(&entity);

        let label = ui.selectable_label(is_selected, entity_name)
            .context_menu(|ui| {
                if ui.button("Add child").clicked() {
                    let new_id = commands.spawn_empty().id();
                    commands.entity(entity).add_child(new_id);
                }
                if ui.button("Delete").clicked() {
                    commands.entity(entity).despawn();
                    selected.list.remove(&entity);
                    ui.close_menu();
                }
            });

        if label.clicked() {
            if !is_selected {
                if !ui.input(|i| i.modifiers.shift) {
                    selected.list.clear();
                }
                selected.list.insert(entity);
            } else {
                selected.list.remove(&entity);
            }
        }
        
        if let Some(children) = children {
            for child in children.iter() {
                draw_entity(commands, ui, query, *child, selected);
            }
        }
    });
}