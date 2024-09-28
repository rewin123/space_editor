use bevy::{prelude::*, sprite::Mesh2dHandle};
use bevy_scene_hook::SceneHook;
#[cfg(feature = "editor")]
use space_shared::toast::ToastMessage;
use space_shared::PrefabMarker;

use crate::prelude::ChildPath;

use super::component::*;

#[derive(Component)]
pub struct WantChildPath;

/// System responsible for spawning GLTF objects in the scene
pub fn spawn_scene(
    mut commands: Commands,
    prefabs: Query<
        (
            Entity,
            &GltfPrefab,
            Option<&Children>,
            Option<&Visibility>,
            Option<&Transform>,
            Option<&SceneAutoChild>,
        ),
        Changed<GltfPrefab>,
    >,
    auto_children: Query<&SceneAutoChild>,
    asset_server: Res<AssetServer>,
) {
    for (e, prefab, children, visibility, transform, auto_child) in prefabs.iter() {
        if let Some(children) = children {
            for e in children {
                if auto_children.contains(*e) {
                    commands.entity(*e).despawn_recursive();
                }
            }
        }

        let is_auto_child = auto_child.is_some();

        commands.entity(e).insert(SceneAutoRoot);

        commands
            .entity(e)
            .insert(asset_server.load::<Scene>(format!("{}#{}", &prefab.path, &prefab.scene)))
            .insert(SceneHook::new(move |e, cmd| {
                if e.contains::<SceneAutoRoot>() {
                    cmd.insert(WantChildPath);
                } else if is_auto_child {
                    cmd.insert(SceneAutoChild);
                } else {
                    cmd.insert((SceneAutoChild, PrefabMarker));
                }
            }));

        if visibility.is_none() {
            commands.entity(e).insert(VisibilityBundle::default());
        }
        if transform.is_none() {
            commands.entity(e).insert(TransformBundle::default());
        }
    }
}

pub fn create_child_path(
    mut commands: Commands,
    prefabs: Query<(Entity, &GltfPrefab), With<WantChildPath>>,
    children: Query<&Children>,
) {
    for (e, _) in prefabs.iter() {
        recursive_path(&mut commands, &children, e, vec![]);
        commands.entity(e).remove::<WantChildPath>();
    }
}

fn recursive_path(
    commands: &mut Commands,
    q_children: &Query<&Children>,
    entity: Entity,
    path: Vec<usize>,
) {
    commands.entity(entity).insert(ChildPath(path.clone()));

    if let Ok(children) = q_children.get(entity) {
        for (i, child_entity) in children.iter().enumerate() {
            let mut child_path = path.clone();
            child_path.push(i);

            recursive_path(commands, q_children, *child_entity, child_path);
        }
    }
}

/// System to sync [`Mesh`] and [`MeshPrimitivePrefab`]
pub fn sync_mesh(
    mut commands: Commands,
    query: Query<(Entity, &MeshPrimitive3dPrefab), Changed<MeshPrimitive3dPrefab>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (e, prefab) in query.iter() {
        let mesh = meshes.add(prefab.to_mesh());
        commands.entity(e).insert(mesh);
    }
}

/// System to sync [`StandardMaterial`] and [`MaterialPrefab`]
pub fn sync_material(
    mut commands: Commands,
    query: Query<(Entity, &MaterialPrefab), Changed<MaterialPrefab>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for (e, prefab) in query.iter() {
        let mat = materials.add(prefab.to_material(&asset_server));
        commands.entity(e).insert(mat);
    }
}

/// System to sync [`Mesh`] and [`MeshPrimitive2dPrefab`]
pub fn sync_2d_mesh(
    mut commands: Commands,
    query: Query<(Entity, &MeshPrimitive2dPrefab), Changed<MeshPrimitive2dPrefab>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (e, prefab) in query.iter() {
        let mesh = bevy::sprite::Mesh2dHandle(meshes.add(prefab.to_mesh()));
        commands.entity(e).insert(mesh);
    }
}

/// System to sync [`ColorMaterial`] and [`ColorMaterialPrefab`]
pub fn sync_2d_material(
    mut commands: Commands,
    query: Query<(Entity, &ColorMaterialPrefab), Changed<ColorMaterialPrefab>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
) {
    for (e, prefab) in query.iter() {
        let mat = materials.add(prefab.to_material(&asset_server));
        commands.entity(e).insert(mat);
    }
}

/// remove mesh handle if prefab struct was removed in editor states
pub fn editor_remove_mesh(
    mut commands: Commands,
    mut query: RemovedComponents<MeshPrimitive3dPrefab>,
) {
    for e in query.read() {
        if let Some(mut cmd) = commands.get_entity(e) {
            cmd.remove::<Handle<Mesh>>();
            info!("Removed mesh handle for {:?}", e);
        }
    }
}

/// remove mesh handle if prefab struct was removed in editor states
pub fn editor_remove_mesh_2d(
    mut commands: Commands,
    mut query: RemovedComponents<MeshPrimitive2dPrefab>,
) {
    for e in query.read() {
        if let Some(mut cmd) = commands.get_entity(e) {
            cmd.remove::<Handle<Mesh>>().remove::<Mesh2dHandle>();
            info!("Removed mesh handle for {:?}", e);
        }
    }
}

/// System to sync [`SpriteBundle`] and [`SpriteTexture`]
pub fn sync_sprite_texture(
    mut commands: Commands,
    query: Query<(Entity, &SpriteTexture), Changed<SpriteTexture>>,
    asset_server: Res<AssetServer>,
) {
    for (e, prefab) in query.iter() {
        if let Some(sprite) = prefab.to_sprite(&asset_server) {
            commands.entity(e).insert(sprite);
        }
    }
}

/// System to sync [`SpriteBundle`] and [`SpriteTexture`]
pub fn sync_spritesheet(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &SpritesheetTexture,
            &AnimationIndicesSpriteSheet,
            &mut TextureAtlasPrefab,
            &AnimationClipName,
        ),
        Or<(
            Changed<SpritesheetTexture>,
            Changed<AnimationIndicesSpriteSheet>,
            Changed<TextureAtlasPrefab>,
            Changed<AnimationClipName>,
        )>,
    >,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    for (e, prefab, clips, mut texture_atlas, clip_name) in query.iter_mut() {
        if let Some(atlas) =
            texture_atlas.to_texture_atlas(prefab, &mut texture_atlases, &asset_server)
        {
            if let Some(clip) = clips.clips.get(&clip_name.name) {
                commands
                    .entity(e)
                    .insert(SpriteBundle {
                        texture: texture_atlas.clone().texture.unwrap_or_default(),
                        transform: Transform::from_scale(Vec3::splat(6.0)),
                        ..default()
                    })
                    .insert(TextureAtlas {
                        layout: atlas,
                        index: clip.first,
                    });
            };
        }
    }
}

/// Spawn system on enter to [`EditorState::Game`] state
///
/// [`EditorState::Game`]: crate::EditorState::Game
pub fn spawn_player_start(
    mut commands: Commands,
    query: Query<(Entity, &PlayerStart)>,
    asset_server: Res<AssetServer>,
    #[cfg(feature = "editor")] mut toast: EventWriter<ToastMessage>,
) {
    for (e, prefab) in query.iter() {
        let msg = format!("Spawning player start: {:?} with \"{}\"", e, &prefab.prefab);
        #[cfg(feature = "editor")]
        toast.send(ToastMessage::new(
            &msg,
            space_shared::toast::ToastKind::Info,
        ));
        info!(msg);
        let child = commands
            .spawn(DynamicSceneBundle {
                scene: asset_server.load(prefab.prefab.to_string()),
                ..default()
            })
            .id();
        commands.entity(e).add_child(child);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::scene::ScenePlugin;

    #[test]
    fn sync_cube_mesh() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(MeshPrimitive3dPrefab::Cube(3.));
            })
            .init_resource::<Assets<Mesh>>()
            .add_systems(Update, sync_mesh);

        app.update();

        let mut query = app
            .world_mut()
            .query::<(&MeshPrimitive3dPrefab, &Handle<Mesh>)>();
        assert_eq!(query.iter(&app.world_mut()).count(), 1);
    }

    #[test]
    fn sync_cube_material() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn((MeshPrimitive3dPrefab::Cube(3.), MaterialPrefab::default()));
            })
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<StandardMaterial>>()
            .add_systems(Update, sync_material);

        app.update();

        let mut query = app
            .world_mut()
            .query::<(&MaterialPrefab, &Handle<StandardMaterial>)>();
        assert_eq!(query.iter(&app.world_mut()).count(), 1);
    }

    #[test]
    fn sync_2d_circle_mesh() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(MeshPrimitive2dPrefab::Circle(CirclePrefab { r: 3.0 }));
            })
            .init_resource::<Assets<Mesh>>()
            .add_systems(Update, sync_2d_mesh);

        app.update();

        let mut query = app
            .world_mut()
            .query::<(&MeshPrimitive2dPrefab, &Mesh2dHandle)>();
        assert_eq!(query.iter(&app.world_mut()).count(), 1);
    }

    #[test]
    fn sync_2d_material_prefab() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn((
                    MeshPrimitive2dPrefab::Circle(CirclePrefab { r: 3.0 }),
                    ColorMaterialPrefab::default(),
                ));
            })
            .init_resource::<Assets<ColorMaterial>>()
            .add_systems(Update, sync_2d_material);

        app.update();

        let mut query = app
            .world_mut()
            .query::<(&ColorMaterialPrefab, &Handle<ColorMaterial>)>();
        assert_eq!(query.iter(&app.world_mut()).count(), 1);
    }

    #[test]
    fn remove_synced_2d_circle_mesh() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(MeshPrimitive2dPrefab::Circle(CirclePrefab { r: 3.0 }));
            })
            .init_resource::<Assets<Mesh>>()
            .add_systems(Update, sync_2d_mesh)
            .add_systems(Update, editor_remove_mesh_2d);

        app.update();

        let mut query = app
            .world_mut()
            .query::<(&MeshPrimitive2dPrefab, &Mesh2dHandle)>();
        assert_eq!(query.iter(&app.world_mut()).count(), 1);

        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<MeshPrimitive2dPrefab>>();
        let entity = query.single(&app.world_mut());
        app.world_mut()
            .entity_mut(entity)
            .remove::<MeshPrimitive2dPrefab>();

        app.update();
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<Mesh2dHandle>>();
        assert_eq!(query.iter(&app.world_mut()).count(), 0);
    }

    #[test]
    fn remove_synced_cube_mesh() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .add_systems(Startup, |mut commands: Commands| {
                commands.spawn(MeshPrimitive3dPrefab::Cube(3.));
            })
            .init_resource::<Assets<Mesh>>()
            .add_systems(Update, sync_mesh)
            .add_systems(Update, editor_remove_mesh);

        app.update();

        let mut query = app
            .world_mut()
            .query::<(&MeshPrimitive3dPrefab, &Handle<Mesh>)>();
        assert_eq!(query.iter(&app.world_mut()).count(), 1);

        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<MeshPrimitive3dPrefab>>();
        let entity = query.single(&app.world_mut());
        app.world_mut()
            .entity_mut(entity)
            .remove::<MeshPrimitive3dPrefab>();

        app.update();
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<Handle<Mesh>>>();
        assert_eq!(query.iter(&app.world_mut()).count(), 0);
    }

    #[test]
    fn sync_sprite_texture_prefab() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default(),
        ))
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(SpriteTexture {
                texture: String::from("test_asset.png"),
            });
        })
        .add_systems(Update, sync_sprite_texture)
        .init_resource::<Assets<Image>>();

        app.update();

        let mut query = app.world_mut().query::<(&SpriteTexture, &Sprite)>();
        assert_eq!(query.iter(&app.world_mut()).count(), 1);
    }

    #[test]
    #[cfg(feature = "editor")]
    fn spawns_player_with_prefab() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default(),
            bevy::scene::ScenePlugin,
        ))
        .add_event::<ToastMessage>();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(PlayerStart {
                prefab: String::from("cube.glb#Scene0"),
            });
        })
        .add_systems(Update, spawn_player_start);
        app.update();

        let events = app.world_mut().resource::<Events<ToastMessage>>();
        let mut man_events = events.get_reader();
        let mut events = man_events.read(events);
        let event = events.next().unwrap();

        assert_eq!(event.kind, space_shared::toast::ToastKind::Info);
        assert_eq!(
            event.text,
            "Spawning player start: Entity { index: 0, generation: 1 } with \"cube.glb#Scene0\""
        );

        let mut query = app
            .world_mut()
            .query::<(Entity, &PlayerStart, Option<&Children>)>();
        let mut iter = query.iter(&app.world());
        assert!(iter.next().unwrap().2.is_some());
    }

    #[test]
    fn create_gltf_with_child() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Startup, |mut commands: Commands| {
                let child_1 = commands
                    .spawn(TransformBundle::default())
                    .with_children(|c| {
                        c.spawn(TransformBundle::default());
                    })
                    .id();
                let child_2 = commands
                    .spawn(TransformBundle::default())
                    .with_children(|c| {
                        c.spawn(TransformBundle::default());
                    })
                    .id();

                commands
                    .spawn((
                        GltfPrefab::default(),
                        TransformBundle::default(),
                        WantChildPath,
                    ))
                    .add_child(child_1)
                    .add_child(child_2);
            })
            .add_systems(Update, create_child_path);

        app.update();

        let mut parent_query = app.world_mut().query_filtered::<Entity, (
            Without<WantChildPath>,
            Without<Parent>,
            With<Children>,
        )>();
        assert_eq!(parent_query.iter(&app.world_mut()).count(), 1);

        let possibilities = vec![vec![0], vec![1], vec![0, 0], vec![1, 0], vec![]];
        let mut child_paths = app.world_mut().query::<&ChildPath>();
        child_paths
            .iter(&app.world_mut())
            .for_each(|d| assert!(possibilities.contains(&d.0)));
    }

    #[test]
    fn sync_spritesheet_to_spritesheetbundle() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default(),
        ))
        .init_resource::<Assets<TextureAtlasLayout>>()
        .init_resource::<Assets<Image>>();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((
                SpritesheetTexture {
                    texture: String::from("gabe-idle-run.png"),
                },
                AnimationIndicesSpriteSheet::default(),
                TextureAtlasPrefab::default(),
                AnimationClipName::default(),
            ));
        });
        app.add_systems(Update, sync_spritesheet);

        app.update();

        let mut query = app.world_mut().query::<(&TextureAtlas, &Sprite)>();

        assert_eq!(query.iter(&app.world_mut()).count(), 1);
    }

    #[test]
    fn spawns_gltf() {
        #[derive(Component)]
        struct DespawnTestChild;

        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ScenePlugin::default(),
        ));
        app.add_systems(Startup, |mut commands: Commands| {
            let child = commands.spawn((SceneAutoChild, DespawnTestChild)).id();
            commands
                .spawn(GltfPrefab {
                    path: String::from("low_poly_fighter_2.gltf"),
                    scene: String::from("Scene0"),
                })
                .add_child(child);
        })
        .add_systems(Update, spawn_scene);

        app.update();

        let mut query = app
            .world_mut()
            .query::<(&Handle<Scene>, &SceneAutoRoot, &Visibility, &Transform)>();

        let s = query.single(&app.world());

        assert_eq!(
            s.0.path().unwrap().to_string(),
            "low_poly_fighter_2.gltf#Scene0"
        );

        let mut query = app.world_mut().query::<(Entity, &DespawnTestChild)>();
        assert!(query.get_single(&app.world_mut()).is_err());
    }

    #[test]
    fn spawns_gltf_with_visibility_and_transform() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ScenePlugin::default(),
        ));
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((
                GltfPrefab {
                    path: String::from("low_poly_fighter_2.gltf"),
                    scene: String::from("Scene0"),
                },
                Visibility::Hidden,
                TransformBundle::IDENTITY,
            ));
        })
        .add_systems(Update, spawn_scene);

        app.update();

        let mut query = app
            .world_mut()
            .query::<(&Handle<Scene>, &Visibility, &Transform)>();

        let s = query.single(&app.world());

        assert_eq!(s.1, Visibility::Hidden);
        assert_eq!(s.2, &Transform::IDENTITY);
    }
}
