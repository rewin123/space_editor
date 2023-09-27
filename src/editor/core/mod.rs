pub mod selected;
pub use selected::*;

mod load;
use load::*;

use bevy::prelude::*;

use crate::{prelude::EditorLoader, prefab::save::SaveState, EditorSet, EditorState};


pub struct EditorCore;

impl Plugin for EditorCore {
    fn build(&self, app: &mut App) {
        app.add_event::<EditorEvent>();
        
        app.add_systems(Update, (apply_deferred, load_listener).chain().after(crate::prelude::bot_menu).in_set(EditorSet::Editor));
        app.add_systems(Update, editor_event_listener);
    }
}

#[derive(Event)]
pub enum EditorEvent {
    Load(String),
    Save(String),
    StartGame
}

fn editor_event_listener(
    mut events: EventReader<EditorEvent>,
    mut load_server : ResMut<EditorLoader>,
    assets : Res<AssetServer>,
    mut save_state : ResMut<NextState<SaveState>>,
    mut start_game_state : ResMut<NextState<EditorState>>
) {
    for event in events.iter() {
        match event {
            EditorEvent::Load(path) => {
                    load_server.scene = Some(
                        assets.load(path.to_string())
                    );
            },
            EditorEvent::Save(_path) => {
                save_state.set(SaveState::Save);
            },
            EditorEvent::StartGame => {
                start_game_state.set(EditorState::GamePrepare);
            },
        }
    }
}
