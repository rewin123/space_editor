use bevy::{
    asset::{AssetPath, LoadState},
    ecs::world::CommandQueue,
    gltf::{Gltf, GltfMesh, GltfNode},
    prelude::*,
    utils::HashMap,
};

use space_prefab::component::{AssetMaterial, AssetMesh, MaterialPrefab};
use space_shared::PrefabMarker;

use super::{BackgroundTask, BackgroundTaskStorage};

#[derive(Event)]
/// Event to handle GLTF path
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
        app.add_systems(PreUpdate, (unpack_gltf_event, queue_push, unpack_gltf));

        app.init_resource::<GltfSceneQueue>();

        app.register_type::<GltfHolder>();
    }
}

#[derive(Component, Reflect, Clone, Default)]
#[reflect(Component)]
struct GltfHolder(Handle<Gltf>);

#[derive(Resource, Default)]
struct GltfSceneQueue(Vec<Handle<Gltf>>);

fn unpack_gltf_event(
    mut events: EventReader<EditorUnpackGltf>,
    assets: Res<AssetServer>,
    mut queue: ResMut<GltfSceneQueue>,
    mut background_tasks: ResMut<BackgroundTaskStorage>,
) {
    for event in events.read() {
        let handle = assets.load(event.path.clone());
        background_tasks.tasks.push(BackgroundTask::AssetLoading(
            event.path.clone(),
            handle.clone().untyped(),
        ));
        queue.0.push(handle);
    }
    events.clear();
}

// separated from unpack_gltf for reduce arguments count and ordered unpack
fn queue_push(
    mut queue: ResMut<GltfSceneQueue>,
    mut events: EventWriter<GltfLoaded>,
    assets: Res<AssetServer>,
) {
    if !queue.0.is_empty() && assets.get_load_state(&queue.0[0]) == Some(LoadState::Loaded) {
        events.send(GltfLoaded(queue.0.remove(0)));
    }
}

struct UnpackContext<'a> {
    material_map: &'a HashMap<Handle<StandardMaterial>, usize>,
    mesh_map: &'a HashMap<Handle<GltfMesh>, usize>,
    gltf_meshs: &'a Assets<GltfMesh>,
    gltf_path: &'a AssetPath<'a>,
}

fn unpack_gltf(world: &mut World) {
    let loaded_scenes = {
        let Some(mut events) = world.get_resource_mut::<Events<GltfLoaded>>() else {
            return;
        };
        let mut reader = events.get_reader();
        let loaded = reader.read(&events).cloned().collect::<Vec<GltfLoaded>>();
        events.clear();
        loaded
    };

    let mut command_queue = CommandQueue::default();
    for gltf in loaded_scenes.iter() {
        let handle: Handle<Gltf> = gltf.0.clone();
        let gltf_path = if let Some(path) = handle.path() {
            path.clone()
        } else {
            continue;
        };
        info!("Path: {:?}", &gltf_path);

        let Some(gltf) = world
            .get_resource::<Assets<Gltf>>()
            .and_then(|gltfs| gltfs.get(&gltf.0))
        else {
            world.send_event(space_shared::toast::ToastMessage::new(
                "Gltf asset not found or empty",
                space_shared::toast::ToastKind::Error,
            ));
            continue;
        };

        let mut commands = Commands::new(&mut command_queue, world);

        let Some(gltf_nodes) = world.get_resource::<Assets<GltfNode>>() else {
            world.send_event(space_shared::toast::ToastMessage::new(
                "Gltf Node asset not found",
                space_shared::toast::ToastKind::Error,
            ));
            continue;
        };
        let Some(gltf_meshs) = world.get_resource::<Assets<GltfMesh>>() else {
            world.send_event(space_shared::toast::ToastMessage::new(
                "Gltf Mesh asset not found",
                space_shared::toast::ToastKind::Error,
            ));
            continue;
        };
        let Some(scenes) = world.get_resource::<Assets<Scene>>() else {
            world.send_event(space_shared::toast::ToastMessage::new(
                "Scene asset not found",
                space_shared::toast::ToastKind::Error,
            ));
            continue;
        };

        let mut mesh_map = HashMap::new();
        for idx in 0..gltf.meshes.len() {
            mesh_map.insert(gltf.meshes[idx].clone(), idx);
        }

        let mut material_map = HashMap::new();
        for idx in 0..gltf.materials.len() {
            info!("Material: {:?}", &gltf.materials[idx]);
            material_map.insert(gltf.materials[idx].clone(), idx);
        }

        for idx in 0..gltf.scenes.len() {
            let Some(scene) = scenes.get(&gltf.scenes[idx]) else {
                continue;
            };

            //find roots nodes
            let mut roots = vec![];
            for e in scene.world.iter_entities() {
                if !e.contains::<Parent>() && e.contains::<Children>() {
                    let Some(children) = e.get::<Children>() else {
                        continue;
                    };
                    for child in children.iter() {
                        if let Some(name) = scene.world.entity(*child).get::<Name>() {
                            info!("Name: {:?}", &name);
                            if let Some(node_handle) = gltf.named_nodes.get(name.as_str()) {
                                if let Some(node) = gltf_nodes.get(node_handle) {
                                    roots.push(node.clone());
                                }
                            }
                        }
                    }
                }
            }

            info!("Roots: {:?}", &roots);

            let ctx = UnpackContext {
                material_map: &material_map,
                mesh_map: &mesh_map,
                gltf_meshs,
                gltf_path: &gltf_path,
            };

            for root in roots.iter() {
                spawn_node(&mut commands, root, gltf, &ctx);
            }
        }

        break;
    }

    command_queue.apply(world);
}

fn spawn_node(
    commands: &mut Commands,
    node: &GltfNode,
    _gltf: &Gltf,
    ctx: &UnpackContext<'_>,
) -> Entity {
    let id = commands
        .spawn((
            SpatialBundle {
                transform: node.transform,
                ..default()
            },
            PrefabMarker,
        ))
        .id();

    if let Some(handle) = &node.mesh {
        if let Some(mesh) = ctx.gltf_meshs.get(handle) {
            if mesh.primitives.len() == 1 {
                commands.entity(id).insert(AssetMesh {
                    path: format!(
                        "{}#Mesh{}/Primitive{}",
                        ctx.gltf_path.path().display(),
                        ctx.mesh_map.get(handle).unwrap(),
                        0
                    ),
                });

                if let Some(material_handle) = &mesh.primitives[0].material {
                    if let Some(idx) = ctx.material_map.get(material_handle) {
                        commands.entity(id).insert(AssetMaterial {
                            path: format!("{}#Material{}", ctx.gltf_path.path().display(), idx),
                        });
                    } else {
                        commands.entity(id).insert(MaterialPrefab::default());
                    }
                } else {
                    commands.entity(id).insert(MaterialPrefab::default());
                }
            } else {
                commands.entity(id).with_children(|parent| {
                    for idx in 0..mesh.primitives.len() {
                        let mut id = parent.spawn((
                            SpatialBundle::default(),
                            AssetMesh {
                                path: format!(
                                    "{}#Mesh{}/Primitive{}",
                                    ctx.gltf_path.path().display(),
                                    ctx.mesh_map.get(handle).unwrap(),
                                    idx
                                ),
                            },
                            PrefabMarker,
                        ));

                        if let Some(material_handle) = &mesh.primitives[idx].material {
                            if let Some(idx) = ctx.material_map.get(material_handle) {
                                id.insert(AssetMaterial {
                                    path: format!(
                                        "{}#Material{}",
                                        ctx.gltf_path.path().display(),
                                        idx
                                    ),
                                });
                            } else {
                                id.insert(MaterialPrefab::default());
                            }
                        } else {
                            id.insert(MaterialPrefab::default());
                        }
                    }
                });
            }
        }
    }

    for child in &node.children {
        let child_id = spawn_node(commands, child, _gltf, ctx);
        commands.entity(id).add_child(child_id);
    }

    id
}
