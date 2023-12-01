pub mod selected;
use bot_menu::EditorLoader;
pub use selected::*;

mod load;
use load::*;

pub mod tool;
pub use tool::*;

pub mod task_storage;
pub use task_storage::*;

pub mod hotkeys;
pub use hotkeys::*;

#[cfg(feature = "persistance_editor")]
pub mod persistance;
#[cfg(feature = "persistance_editor")]
pub use persistance::*;

pub mod gltf_unpack;

use bevy::prelude::*;

use prefab::prefab::save::{SaveConfig, SaveState};
use shared::*;
use undo::{AppAutoUndo, UndoPlugin};

use super::prelude::bot_menu;

pub struct EditorCore;

impl Plugin for EditorCore {
    fn build(&self, app: &mut App) {
        app.add_plugins(gltf_unpack::UnpackGltfPlugin);

        #[cfg(feature = "persistance_editor")]
        app.add_plugins(PersistancePlugin);

        app.add_plugins(BackgroundTaskStoragePlugin);
        app.add_plugins(UndoPlugin);

        app.add_event::<EditorEvent>();

        app.init_resource::<PrefabMemoryCache>();

        app.add_systems(
            Update,
            (apply_deferred, load_listener)
                .chain()
                .after(bot_menu)
                .in_set(EditorSet::Editor),
        );
        app.add_systems(Update, editor_event_listener);

        app.auto_reflected_undo::<Parent>();
        app.auto_reflected_undo::<Children>();
        app.auto_undo::<PrefabMarker>();
    }
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
                }
                EditorPrefabPath::MemoryCahce => {
                    load_server.scene = cache.scene.clone();
                }
            },
            EditorEvent::Save(path) => {
                save_config.path = Some(path.clone());
                save_state.set(SaveState::Save);
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
