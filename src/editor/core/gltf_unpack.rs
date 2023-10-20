use std::arch::x86_64::_MM_FROUND_NEARBYINT;

use bevy::{prelude::*, gltf::{Gltf, GltfNode, GltfMesh}, asset::LoadState, ecs::{entity::EntityMap, system::CommandQueue}, utils::HashMap};

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

        app.register_type::<GltfHolder>();
    }
}

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component)]
struct GltfHolder(Handle<Gltf>);

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
    events.clear();
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
        let loaded = reader.iter(&events).map(|e| e.clone()).collect::<Vec<GltfLoaded>>();
        events.clear();
        loaded
    };

    let default_material = world.resource_mut::<Assets<StandardMaterial>>().add(StandardMaterial::default());

    let mut command_queue = CommandQueue::default();
    for gltf in loaded_scenes.iter() {
        let handle: Handle<Gltf> = gltf.0.clone();
        let gltf_path = world.resource::<AssetServer>().get_handle_path(&handle).unwrap();
        info!("Path: {:?}", &gltf_path);

        let Some(gltf) = world.resource::<Assets<Gltf>>().get(&gltf.0) else {
            continue;
        };

        // let mut spawned = vec![];

        let mut commands = Commands::new(&mut command_queue, &world);

        let gltf_nodes = world.resource::<Assets<GltfNode>>();
        let gltf_meshs = world.resource::<Assets<GltfMesh>>();
        let scenes = world.resource::<Assets<Scene>>();

        let mut mesh_map = HashMap::new();
        for idx in 0..gltf.meshes.len() {
            mesh_map.insert(gltf.meshes[idx].clone(), idx);
        }

        let mut spawned = vec![];

        for idx in 0..gltf.scenes.len() {
            let Some(scene) = scenes.get(&gltf.scenes[idx]) else {
                continue;
            };

            let mut new_scene = Scene::new(World::default());
            // new_scene.world.insert_resource(world.resource::<AppTypeRegistry>().clone());
            // scene.write_to_world_with(&mut new_scene.world, world.resource::<AppTypeRegistry>());
            // let dyn_scene = DynamicScene::from_scene(&new_scene);
            // for e in dyn_scene.entities {
            //     for c in e.components {
            //         info!("{}", c.type_name());
            //     }
            // }
            // for e in scene.world.iter_entities() {
            //     if let Some(mesh) = e.get::<Handle<Mesh>>() {
            //         info!("Mesh: {:?} : {:?}", mesh, world.resource::<AssetServer>().get_handle_path(mesh));
            //     }
            // }
            // info!("Scene: {:?}", dyn_scene.entities);
        }

        for idx in 0..gltf.nodes.len() {
            spawned.push(commands.spawn((SpatialBundle::default(), PrefabMarker)).id());
        }

        for idx in 0..gltf.nodes.len() {
            let Some(node) = gltf_nodes.get(&gltf.nodes[idx]) else {
                continue;
            };

            commands.entity(spawned[idx]).insert(node.transform.clone());

            
            if let Some(mesh_handle) = &node.mesh {
                let mesh_idx = mesh_map.get(mesh_handle).unwrap();
                if let Some(mesh) = gltf_meshs.get(mesh_handle) {
                    for primitive_idx in 0..mesh.primitives.len() {
                        let id = commands.spawn((
                            PbrBundle {
                                mesh: world.resource::<AssetServer>().load(format!("{}#Mesh{}/Primitive{}", gltf_path.path().display(), mesh_idx, primitive_idx)),
                                material : default_material.clone(),
                                ..default()
                            },
                            PrefabMarker
                        )).id();
                        commands.entity(spawned[idx]).add_child(id);
                    }
                }
            }
        }

        break;
    }

    command_queue.apply(world);
}