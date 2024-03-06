#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

pub mod hotkeys;
mod load;
pub mod selected;
pub mod task_storage;
pub mod toast;

pub mod prelude {
    pub use super::*;
    pub use super::{hotkeys::*, load::*, selected::*, task_storage::*};
    pub use space_undo;
}

pub mod gltf_unpack;

use bevy::prelude::*;

use prelude::load_listener;
use space_prefab::save::{SaveConfig, SaveState};
use space_shared::*;
use space_undo::AppAutoUndo;
use task_storage::{BackgroundTask, BackgroundTaskStorage, BackgroundTaskStoragePlugin};

pub struct EditorCore;

impl Plugin for EditorCore {
    fn build(&self, app: &mut App) {
        app.add_plugins(gltf_unpack::UnpackGltfPlugin);

        #[cfg(feature = "persistence_editor")]
        app.add_plugins(space_persistence::PersistencePlugin);

        app.add_plugins(BackgroundTaskStoragePlugin);

        app.configure_sets(Update, EditorLoadSet.in_set(EditorSet::Editor));

        app.add_event::<EditorEvent>();

        app.init_resource::<PrefabMemoryCache>();

        app.add_systems(
            Update,
            (apply_deferred, load_listener)
                .chain()
                .in_set(EditorLoadSet),
        );
        app.add_systems(Update, editor_event_listener);

        app.auto_reflected_undo::<Parent>();
        app.auto_reflected_undo::<Children>();
        app.auto_undo::<PrefabMarker>();
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct EditorLoadSet;

#[derive(Resource, Default, Clone)]
pub struct EditorLoader {
    pub scene: Option<Handle<DynamicScene>>,
}

fn editor_event_listener(
    mut events: EventReader<EditorEvent>,
    mut load_server: ResMut<EditorLoader>,
    assets: Res<AssetServer>,
    mut save_state: ResMut<NextState<SaveState>>,
    mut save_config: ResMut<SaveConfig>,
    mut start_game_state: ResMut<NextState<EditorState>>,
    cache: ResMut<PrefabMemoryCache>,
    mut gltf_events: EventWriter<gltf_unpack::EditorUnpackGltf>,
    mut background_tasks: ResMut<BackgroundTaskStorage>,
) {
    for event in events.read() {
        match event {
            EditorEvent::Load(path) => match path {
                EditorPrefabPath::File(path) => {
                    let handle = assets.load(path.to_string());
                    background_tasks.tasks.push(BackgroundTask::AssetLoading(
                        path.to_string(),
                        handle.clone().untyped(),
                    ));
                    load_server.scene = Some(handle);
                    info!("Loading prefab by editor event from file {}", path);
                }
                EditorPrefabPath::MemoryCache => {
                    load_server.scene = cache.scene.clone();
                    info!("Loading prefab by editor event from memory cache");
                }
            },
            EditorEvent::Save(path) => {
                save_config.path = Some(path.clone());
                save_state.set(SaveState::Save);
                info!("Saving scene to {:?}", path);
            }
            EditorEvent::StartGame => {
                start_game_state.set(EditorState::GamePrepare);
            }
            EditorEvent::LoadGltfAsPrefab(path) => {
                gltf_events.send(gltf_unpack::EditorUnpackGltf { path: path.clone() })
            }
        }
    }
}
