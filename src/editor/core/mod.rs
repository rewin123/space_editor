pub mod selected;
pub use selected::*;

mod load;
use load::*;

pub mod tool;
pub use tool::*;

pub mod gltf_unpack;
pub mod settings;

use bevy::prelude::*;

use crate::{
    prefab::save::{SaveConfig, SaveState},
    prelude::EditorLoader,
    EditorSet, EditorState,
};

pub struct EditorCore;

impl Plugin for EditorCore {
    fn build(&self, app: &mut App) {
        app.add_plugins(gltf_unpack::UnpackGltfPlugin);

        app.add_event::<EditorEvent>();

        app.init_resource::<PrefabMemoryCache>();

        app.add_systems(
            Update,
            (apply_deferred, load_listener)
                .chain()
                .after(crate::prelude::bot_menu)
                .in_set(EditorSet::Editor),
        );
        app.add_systems(Update, editor_event_listener);
    }
}

#[derive(Resource, Default)]
pub struct PrefabMemoryCache {
    pub scene: Option<Handle<DynamicScene>>,
}

#[derive(Clone, Debug)]
pub enum EditorPrefabPath {
    File(String),
    MemoryCahce,
}

#[derive(Event)]
pub enum EditorEvent {
    Load(EditorPrefabPath),
    Save(EditorPrefabPath),
    LoadGltfAsPrefab(String),
    StartGame,
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
) {
    for event in events.read() {
        match event {
            EditorEvent::Load(path) => match path {
                EditorPrefabPath::File(path) => {
                    load_server.scene = Some(assets.load(path.to_string()))
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
