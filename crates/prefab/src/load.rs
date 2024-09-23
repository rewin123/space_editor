use bevy::prelude::*;
use bevy_scene_hook::SceneHook;
use space_shared::PrefabMarker;

use crate::prelude::EditorRegistryExt;

use super::save::ChildrenPrefab;

/// Bundle for spawn prefabs
/// Example
///
/// commands.spawn(PrefabBundle::new("path/to/prefab"));
///
#[derive(Default, Bundle)]
pub struct PrefabBundle {
    loader: PrefabLoader,
    transform: Transform,
    global_transform: GlobalTransform,

    visibility: Visibility,
    computed_visibility: ViewVisibility,
    inherited_visibility: InheritedVisibility,
}

impl PrefabBundle {
    /// Create new prefab bundle from path to prefab file
    pub fn new(path: &str) -> Self {
        Self {
            loader: PrefabLoader {
                path: path.to_string(),
            },
            ..default()
        }
    }
}

/// Plugin for loading prefabs
pub struct LoadPlugin;

/// Marks all child of prefab to correct delete them when prefab is deleted
#[derive(Component)]
pub struct PrefabAutoChild;

impl Plugin for LoadPlugin {
    #[cfg(not(tarpaulin_include))]
    fn build(&self, app: &mut App) {
        app.editor_registry::<PrefabLoader>();

        app.add_systems(
            Update,
            load_prefab.after(bevy_scene_hook::Systems::SceneHookRunner),
        );
        app.add_systems(
            Update,
            conflict_resolve
                .after(bevy_scene_hook::Systems::SceneHookRunner)
                .before(load_prefab),
        );
        app.add_systems(Update, auto_children);
    }
}

/// This component is mark that prefab should be loaded
#[derive(Component, Default, Reflect, Clone)]
#[reflect(Component)]
pub struct PrefabLoader {
    pub path: String,
}

/// System responsible for loading prefabs
fn load_prefab(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &PrefabLoader,
            Option<&Children>,
            Option<&Transform>,
            Option<&Visibility>,
        ),
        Changed<PrefabLoader>,
    >,
    auto_children: Query<Entity, With<PrefabAutoChild>>,
    assets: ResMut<AssetServer>,
) {
    for (e, l, children, tr, vis) in query.iter() {
        if tr.is_none() {
            commands
                .entity(e)
                .insert((Transform::default(), GlobalTransform::default()));
        }
        if vis.is_none() {
            commands.entity(e).insert(VisibilityBundle::default());
        }

        //remove old scene
        if let Some(children) = children {
            for child in children {
                if auto_children.contains(*child) {
                    commands.entity(*child).despawn_recursive();
                }
            }
            commands.entity(e).clear_children();
        }

        let scene: Handle<DynamicScene> = assets.load(&l.path);

        let id = commands
            .spawn(DynamicSceneBundle { scene, ..default() })
            .insert(SceneHook::new(move |_e, cmd| {
                cmd.insert(PrefabAutoChild);
            }))
            .insert(PrefabAutoChild)
            .id();

        commands.entity(e).push_children(&[id]);
    }
}

fn conflict_resolve(
    mut commands: Commands,
    query: Query<Entity, (With<PrefabAutoChild>, With<PrefabMarker>)>,
) {
    for e in query.iter() {
        commands.entity(e).remove::<PrefabMarker>();
    }
}

fn auto_children(
    mut commands: Commands,
    query: Query<(Entity, &ChildrenPrefab)>,
    existing_entity: Query<Entity>,
) {
    for (e, children) in query.iter() {
        let mut cmds = commands.entity(e);
        for child in children.0.iter() {
            if existing_entity.contains(*child) {
                cmds.add_child(*child);
            }
        }
        cmds.remove::<ChildrenPrefab>();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn new_prefab_bundler() {
        let bundler = PrefabBundle::new("path");

        assert_eq!(bundler.loader.path, "path");
    }

    #[test]
    fn conflict_resolver_only_one_prefab_component_allowed() {
        let mut app = App::new();

        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((PrefabAutoChild, PrefabMarker));
            commands.spawn(PrefabAutoChild);
        })
        .add_systems(Update, conflict_resolve);

        app.update();

        let mut query = app
            .world_mut()
            .query_filtered::<Entity, (With<PrefabAutoChild>, With<PrefabMarker>)>();
        assert_eq!(query.iter(&app.world()).count(), 0);

        let mut query = app
            .world_mut()
            .query_filtered::<Entity, With<PrefabAutoChild>>();
        assert_eq!(query.iter(&app.world()).count(), 2);
    }
}
