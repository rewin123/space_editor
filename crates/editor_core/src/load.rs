use bevy::{ecs::entity::EntityHashMap, prelude::*};
use space_shared::{toast::ToastMessage, *};

use crate::EditorLoader;

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

    let mut query = world.query_filtered::<(Entity, Option<&Name>), With<PrefabMarker>>();
    let mark_to_delete: Vec<_> = query
        .iter(world)
        .map(|(e, name)| (e, name.cloned()))
        .collect();
    for (entity, name) in mark_to_delete {
        let mut despawned = false;
        if let Some(e) = world.get_entity_mut(entity) {
            e.despawn_recursive();
            despawned = true;
        }

        if despawned {
            world.send_event(ToastMessage::new(
                &if name.is_some() {
                    format!("Despawning {}: {:?}", name.unwrap(), entity)
                } else {
                    format!("Despawning {:?}", entity)
                },
                egui_toast::ToastKind::Warning,
            ));
        }
    }

    for entity in &mut prefab.entities {
        entity.components.push(Box::new(PrefabMarker));
    }

    let mut map = EntityHashMap::default();
    let res = prefab.write_to_world(world, &mut map);
    match res {
        Ok(_) => {
            world.send_event(ToastMessage::new(
                "Prefab loaded successfully",
                egui_toast::ToastKind::Success,
            ));
        }
        Err(err) => {
            world.send_event(ToastMessage::new(
                &format!("Failed to create scene:\n{err}"),
                egui_toast::ToastKind::Error,
            ));
            bevy::log::error!("{}", err)
        }
    }
}
