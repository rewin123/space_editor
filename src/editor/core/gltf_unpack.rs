

use bevy::{
    asset::{AssetPath, LoadState},
    ecs::{system::CommandQueue},
    gltf::{Gltf, GltfMesh, GltfNode},
    prelude::*,
    utils::{HashMap},
};

use crate::{
    prefab::component::{AssetMaterial, AssetMesh, MaterialPrefab},
    PrefabMarker,
};

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
        app.add_systems(PreUpdate, (unpack_gltf_event, queue_push, unpack_gltf));

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
    mut events: EventReader<EditorUnpackGltf>,
    assets: Res<AssetServer>,
    mut queue: ResMut<GltfSceneQueue>,
) {
    for event in events.iter() {
        queue.0.push(assets.load(event.path.clone()));
    }
    events.clear();
}

// separated from unpack_gltf for reduce arguments count and ordered unpack
fn queue_push(
    mut queue: ResMut<GltfSceneQueue>,
    mut events: EventWriter<GltfLoaded>,
    assets: Res<AssetServer>,
) {
    if !queue.0.is_empty() && assets.get_load_state(&queue.0[0]) == LoadState::Loaded {
        events.send(GltfLoaded(queue.0.remove(0)));
    }
}

struct UnpackContext<'a> {
    material_map: &'a HashMap<Handle<StandardMaterial>, usize>,
    mesh_map: &'a HashMap<Handle<GltfMesh>, usize>,
    gltf_meshs: &'a Assets<GltfMesh>,
    default_material: Handle<StandardMaterial>,
    gltf_path: &'a AssetPath<'a>,
}

fn unpack_gltf(world: &mut World) {
    let loaded_scenes = {
        let mut events = world.resource_mut::<Events<GltfLoaded>>();
        let mut reader = events.get_reader();
        let loaded = reader
            .iter(&events).cloned()
            .collect::<Vec<GltfLoaded>>();
        events.clear();
        loaded
    };

    let default_material = world
        .resource_mut::<Assets<StandardMaterial>>()
        .add(StandardMaterial::default());

    let mut command_queue = CommandQueue::default();
    for gltf in loaded_scenes.iter() {
        let handle: Handle<Gltf> = gltf.0.clone();
        let gltf_path = world
            .resource::<AssetServer>()
            .get_handle_path(&handle)
            .unwrap();
        info!("Path: {:?}", &gltf_path);

        let Some(gltf) = world.resource::<Assets<Gltf>>().get(&gltf.0) else {
            continue;
        };

        let mut commands = Commands::new(&mut command_queue, world);

        let gltf_nodes = world.resource::<Assets<GltfNode>>();
        let gltf_meshs = world.resource::<Assets<GltfMesh>>();
        let scenes = world.resource::<Assets<Scene>>();

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
                    let children = e.get::<Children>().unwrap();
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
                default_material: default_material.clone(),
                gltf_path: &gltf_path,
            };

            for root in roots.iter() {
                spawn_node(&mut commands, root, gltf, &ctx);
            }

            // for e in scene.world.iter_entities() {
            //     let new_id = spawned_map[&e.id()];
            //     {
            //         let mut cmds = commands.entity(new_id);

            //         spawned_map.insert(e.id(), cmds.id());

            //         if let Some(transform) = e.get::<Transform>() {
            //             cmds.insert(transform.clone());
            //         }

            //         if let Some(children) = e.get::<Children>() {
            //             for child in children.iter() {
            //                 cmds.add_child(spawned_map[child].clone());
            //             }
            //         }
            //     }

            //     // if let Some(name) = e.get::<Name>() {
            //     //     commands.entity(new_id).insert(name.clone());

            //     //     if let Some(node_handle) = gltf.named_nodes.get(name.as_str()) {
            //     //         if let Some(node) = gltf_nodes.get(node_handle) {
            //     //             //setup mesh
            //     //             if let Some(mesh_handle) = &node.mesh {
            //     //                 if let Some(mesh) = gltf_meshs.get(mesh_handle) {
            //     //                     if mesh.primitives.len() == 1 {
            //     //                         commands.entity(new_id).insert(AssetMesh {
            //     //                             path : format!("{}#Mesh{}/Primitive{}", gltf_path.path().display(), mesh_map.get(mesh_handle).unwrap(), 0),
            //     //                         });

            //     //                         if let Some(material_handle) = &mesh.primitives[0].material {
            //     //                             if let Some(idx) = material_map.get(material_handle) {
            //     //                                 commands.entity(new_id).insert(
            //     //                                     AssetMaterial {
            //     //                                         path : format!("{}#Material{}", gltf_path.path().display(), idx),
            //     //                                     }
            //     //                                 );
            //     //                             } else {
            //     //                                 commands.entity(new_id).insert(AssetMaterial {
            //     //                                     path : format!("{}#MaterialDefault", gltf_path.path().display()),
            //     //                                 });
            //     //                             }
            //     //                         } else {
            //     //                             commands.entity(new_id).insert(MaterialPrefab::default());
            //     //                         }
            //     //                     } else {
            //     //                         //create childs for every primitive
            //     //                         for idx in 0..mesh.primitives.len() {
            //     //                             let id = commands.spawn((
            //     //                                 PbrBundle {
            //     //                                     material : default_material.clone(),
            //     //                                     ..default()
            //     //                                 },
            //     //                                 AssetMesh {
            //     //                                     path : format!("{}#Mesh{}/Primitive{}", gltf_path.path().display(), mesh_map.get(mesh_handle).unwrap(), idx),
            //     //                                 },
            //     //                                 PrefabMarker
            //     //                             )).id();
            //     //                             commands.entity(new_id).add_child(id);

            //     //                             if let Some(material_handle) = &mesh.primitives[idx].material {
            //     //                                 if let Some(idx) = material_map.get(material_handle) {
            //     //                                     commands.entity(new_id).insert(
            //     //                                         AssetMaterial {
            //     //                                             path : format!("{}#Material{}", gltf_path.path().display(), idx),
            //     //                                         }
            //     //                                     );
            //     //                                 } else {
            //     //                                     commands.entity(new_id).insert(MaterialPrefab::default());
            //     //                                 }
            //     //                             } else {
            //     //                                 commands.entity(new_id).insert(MaterialPrefab::default());
            //     //                             }
            //     //                         }
            //     //                     }
            //     //                 }
            //     //             }
            //     //         }
            //     //     }
            //     }
            // }
        }

        // for idx in 0..gltf.nodes.len() {
        //     spawned.push(commands.spawn((SpatialBundle::default(), PrefabMarker)).id());
        // }

        // for idx in 0..gltf.nodes.len() {
        //     let Some(node) = gltf_nodes.get(&gltf.nodes[idx]) else {
        //         continue;
        //     };

        //     commands.entity(spawned[idx]).insert(node.transform.clone());
        //     info!("Node: {:?}", node);

        //     if let Some(mesh_handle) = &node.mesh {
        //         let mesh_idx = mesh_map.get(mesh_handle).unwrap();
        //         if let Some(mesh) = gltf_meshs.get(mesh_handle) {
        //             for primitive_idx in 0..mesh.primitives.len() {
        //                 let id = commands.spawn((
        //                     PbrBundle {
        //                         mesh: world.resource::<AssetServer>().load(format!("{}#Mesh{}/Primitive{}", gltf_path.path().display(), mesh_idx, primitive_idx)),
        //                         material : default_material.clone(),
        //                         ..default()
        //                     },
        //                     PrefabMarker
        //                 )).id();
        //                 commands.entity(spawned[idx]).add_child(id);
        //             }
        //         }
        //     }
        // }

        break;
    }

    command_queue.apply(world);
}

fn spawn_node(
    commands: &mut Commands,
    node: &GltfNode,
    gltf: &Gltf,
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
            }
        }
    }

    for child in &node.children {
        let child_id = spawn_node(commands, child, gltf, ctx);
        commands.entity(id).add_child(child_id);
    }

    id
}
