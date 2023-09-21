

use bevy::{prelude::*, ecs::entity::EntityMap};
use bevy_egui::*;

use crate::{prefab::{save::{SaveState, SaveConfig}, PrefabPlugin}, PrefabMarker, prelude::show_hierarchy, EditorState, EditorSet};

#[derive(Resource, Default, Clone)]
pub struct EditorLoader {
    pub scene : Option<Handle<DynamicScene>>
}

/// Plugin to activate bot menu in editor UI
pub struct BotMenuPlugin;

impl Plugin for BotMenuPlugin {
    fn build(&self, app: &mut App) {

        if !app.is_plugin_added::<PrefabPlugin>() {
            app.add_plugins(PrefabPlugin);
        }
        app.init_resource::<EditorLoader>();

        app.add_systems(Update, bot_menu
            .after(super::inspector::inspect)
            .after(show_hierarchy)
            .in_set(EditorSet::Editor));
        app.add_systems(Update, (apply_deferred, load_listener).chain().after(bot_menu).in_set(EditorSet::Editor));
        app.add_systems(Update, bot_menu_game.in_set(EditorSet::Game));
        app.add_event::<LoadEvent>();
    }
}

#[derive(Event)]
pub struct LoadEvent {
    pub path : String
}

fn bot_menu_game(
    mut ctxs : EguiContexts,
    mut state : ResMut<NextState<EditorState>>
) {
    egui::TopBottomPanel::bottom("bot_panel").show(ctxs.ctx_mut(), |ui| {
        ui.vertical_centered(|ui| {
            if ui.button("⏸").clicked() {
                state.set(EditorState::Editor);
            }
        });
    });
}

fn bot_menu(
    mut commands : Commands,
    mut ctxs : EguiContexts,
    mut save_confg : ResMut<SaveConfig>,
    mut save_state : ResMut<NextState<SaveState>>,
    mut assets : ResMut<AssetServer>,
    mut load_server : ResMut<EditorLoader>,
    mut state : ResMut<NextState<EditorState>>,
    mut events : EventReader<LoadEvent>
) {
    let ctx = ctxs.ctx_mut();
    egui::TopBottomPanel::bottom("bot_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {

            ui.label("Save path:");
            ui.add(egui::TextEdit::singleline(&mut save_confg.path));

            if ui.button("Save").clicked() {
                save_state.set(SaveState::Save);
            }

            if ui.button("Load").clicked() {
                if !save_confg.path.is_empty() {
                    load_server.scene = Some(
                        assets.load(format!("{}.scn.ron",save_confg.path))
                    );
                }
                // TODO: else show notification with information
            }

            if ui.button("▶").clicked() {
                state.set(EditorState::GamePrepare);
            }
        });
    });

    for event in events.iter() {
        save_confg.path = event.path.clone();
        load_server.scene = Some(
            assets.load(format!("{}.scn.ron",save_confg.path))
        );
    } 
    events.clear();
}

fn load_listener(
    world : &mut World
) {
    let app_registry = world.resource::<AppTypeRegistry>().clone();
    let load_server = world.resource::<EditorLoader>().clone();
    let mut prefab;
    {
        let assets = world.resource::<Assets<DynamicScene>>();
        if let Some(scene) = &load_server.scene {
            if let Some(scene) = assets.get(scene) {
                let mut scene = Scene::from_dynamic_scene(scene, &app_registry).unwrap();
                scene.world.insert_resource(app_registry.clone());
                prefab = DynamicScene::from_scene(&scene); //kill me, is it clone() analog for DynamicScene
            } else {
                return;
            }
        } else {
            return;
        }
    }
    world.resource_mut::<EditorLoader>().scene = None;

    let  mut query = world.query_filtered::<Entity, With<PrefabMarker>>();
    let mark_to_delete : Vec<_> = query.iter(&world).collect();
    for entity in mark_to_delete {
        world.entity_mut(entity).despawn_recursive();
    }

    for entity in &mut prefab.entities {

        entity.components.push(
            Box::new(PrefabMarker)
        );
    }

    let mut map = EntityMap::default();
    prefab.write_to_world(world, &mut map);

}
