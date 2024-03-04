use bevy::prelude::*;
use bevy_scene_hook::SceneHook;
use space_shared::{toast::ToastMessage, PrefabMarker};

use super::component::*;

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
    for (e, prefab, children, vis, tr, auto_child) in prefabs.iter() {
        if let Some(children) = children {
            for e in children {
                if auto_children.contains(*e) {
                    commands.entity(*e).despawn_recursive();
                }
            }
        }

        let is_auto_child = auto_child.is_some();

        commands
            .entity(e)
            .insert(asset_server.load::<Scene>(format!("{}#{}", &prefab.path, &prefab.scene)))
            .insert(SceneHook::new(move |_e, cmd| {
                if _e.contains::<SceneAutoRoot>() {
                } else if is_auto_child {
                    cmd.insert(SceneAutoChild);
                } else {
                    cmd.insert(SceneAutoChild).insert(PrefabMarker);
                }
            }));

        commands.entity(e).insert(SceneAutoRoot);

        if vis.is_none() {
            commands.entity(e).insert(VisibilityBundle::default());
        }
        if tr.is_none() {
            commands.entity(e).insert(TransformBundle::default());
        }
    }
}

/// System to sync [`Mesh`] and [`MeshPrimitivePrefab`]
pub fn sync_mesh(
    mut commands: Commands,
    query: Query<(Entity, &MeshPrimitivePrefab), Changed<MeshPrimitivePrefab>>,
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
        let mesh = meshes.add(prefab.to_mesh());
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
    mut query: RemovedComponents<MeshPrimitivePrefab>,
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
            cmd.remove::<Handle<Mesh>>();
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
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    for (e, prefab, clips, mut atlas, clip_name) in query.iter_mut() {
        if let Some(atlas) = atlas.to_texture_atlas(prefab, &mut texture_atlases, &asset_server) {
            if let Some(clip) = clips.clips.get(&clip_name.name) {
                commands.entity(e).insert(SpriteSheetBundle {
                    texture_atlas: atlas,
                    sprite: TextureAtlasSprite::new(clip.first),
                    transform: Transform::from_scale(Vec3::splat(6.0)),
                    ..default()
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
    mut toast: EventWriter<ToastMessage>,
) {
    for (e, prefab) in query.iter() {
        let msg = format!("Spawning player start: {:?} with \"{}\"", e, &prefab.prefab);
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
    #[cfg(feature = "editor")]
    use super::*;
    #[cfg(feature = "editor")]
    use bevy::scene::ScenePlugin;

    #[test]
    #[cfg(feature = "editor")]
    fn spawns_player_with_prefab() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default(),
            ScenePlugin,
        ))
        .add_event::<ToastMessage>();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn(PlayerStart {
                prefab: String::from("cube.glb#Scene0"),
            });
        })
        .add_systems(Update, spawn_player_start);
        app.update();

        let events = app.world.resource::<Events<ToastMessage>>();
        let mut man_events = events.get_reader();
        let mut events = man_events.read(events);
        let event = events.next().unwrap();

        assert_eq!(event.kind, space_shared::toast::ToastKind::Info);
        assert_eq!(
            event.text,
            "Spawning player start: 0v0 with \"cube.glb#Scene0\""
        );

        let mut query = app
            .world
            .query::<(Entity, &PlayerStart, Option<&Children>)>();
        let mut iter = query.iter(&app.world);
        assert!(iter.next().unwrap().2.is_some());
    }
}

// pub fn despawn_player_start(
//     mut commands : Commands,
//     query : Query<Entity, (With<PlayerStart>, With<Handle<Scene>>)>
// ) {
//     for e in query.iter() {

//     }
// }
