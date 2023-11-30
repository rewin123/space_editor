use bevy::{prelude::*, utils::HashMap};
use shared::*;

use crate::editor::prelude::EditorLoader;

pub fn load_listener(world: &mut World) {
    let app_registry = world.resource::<AppTypeRegistry>().clone();
    let load_server = world.resource::<EditorLoader>().clone();
    let mut prefab;
    {
        let assets = world.resource::<Assets<DynamicScene>>();
        if let Some(scene) = &load_server.scene {
            if let Some(scene) = assets.get(scene) {
                let mut scene = Scene::from_dynamic_scene(scene, &app_registry).unwrap();
                scene.world.insert_resource(app_registry);
                prefab = DynamicScene::from_scene(&scene); //kill me, is it clone() analog for DynamicScene
            } else {
                return;
            }
        } else {
            return;
        }
    }
    world.resource_mut::<EditorLoader>().scene = None;

    let mut query = world.query_filtered::<Entity, With<PrefabMarker>>();
    let mark_to_delete: Vec<_> = query.iter(world).collect();
    for entity in mark_to_delete {
        if let Some(e) = world.get_entity_mut(entity) {
            e.despawn_recursive();
        }
    }

    for entity in &mut prefab.entities {
        entity.components.push(Box::new(PrefabMarker));
    }

    let mut map = HashMap::new();
    let res = prefab.write_to_world(world, &mut map);
    match res {
        Ok(_) => { /*some info planned*/ }
        Err(_) => { /*some info planned*/ }
    }
}
