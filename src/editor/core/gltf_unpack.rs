use bevy::{prelude::*, gltf::Gltf, asset::LoadState, ecs::entity::EntityMap};

use crate::PrefabMarker;

#[derive(Event)]
pub struct EditorUnpackGltf {
    pub path: String,
}

#[derive(Event, Clone)]
struct GltfLoaded(Handle<Gltf>);

pub struct UnpackGltfPlugin;

impl Plugin for UnpackGltfPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EditorUnpackGltf>();
        app.add_event::<GltfLoaded>();
        app.add_systems(
            PreUpdate,
            (
                unpack_gltf_event,
                queue_push,
                unpack_gltf
            )
        );

        app.init_resource::<GltfSceneQueue>();
    }
}

#[derive(Component)]
struct NeedUnpackTag;

#[derive(Resource, Default)]
struct GltfSceneQueue(Vec<Handle<Gltf>>);

fn unpack_gltf_event(
    mut events : EventReader<EditorUnpackGltf>,
    assets : Res<AssetServer>,
    mut queue : ResMut<GltfSceneQueue>,
) {
    for event in events.iter() {
        queue.0.push(assets.load(event.path.clone()));
    }
}

// separated from unpack_gltf for reduce arguments count and ordered unpack
fn queue_push(
    mut queue : ResMut<GltfSceneQueue>,
    mut events : EventWriter<GltfLoaded>,
    assets : Res<AssetServer>
) {
    if !queue.0.is_empty() {
        if assets.get_load_state(&queue.0[0]) == LoadState::Loaded {
            events.send(GltfLoaded(queue.0.remove(0)));
        }
    }
}

fn unpack_gltf(
    world : &mut World
) {
    let loaded_scenes = {
        let mut events = world.resource_mut::<Events<GltfLoaded>>();
        let mut reader = events.get_reader();
        reader.iter(&events).map(|e| e.clone()).collect::<Vec<GltfLoaded>>()
    };

    for gltf in loaded_scenes.iter() {
        let scene = {
            let Some(gltf) = world.resource::<Assets<Gltf>>().get(&gltf.0) else {
                continue;
            };


            //always unpack scene 0 if exist
            if gltf.scenes.len() > 0 {
                let scene = gltf.scenes[0].clone();
                let Some(scene) = world.resource::<Assets<Scene>>().get(&scene) else {
                    continue;
                };
                
                let mut typed_scene = Scene::new(World::default());
                typed_scene.world.insert_resource(world.resource::<AppTypeRegistry>().clone());
                scene.write_to_world_with(&mut typed_scene.world, world.resource::<AppTypeRegistry>());


                let entities = typed_scene.world.iter_entities().map(|e| e.id()).collect::<Vec<_>>();
                for e in entities {
                    typed_scene.world.entity_mut(e)
                        .insert(PrefabMarker);
                }

                let mut scene = DynamicScene::from_scene(&typed_scene);
                
                scene
            } else {
                continue;
            }
        };
        let mut entity_map = EntityMap::default();
        scene.write_to_world(world, &mut entity_map);
    }
}