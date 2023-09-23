pub mod selected;
pub use selected::*;

pub mod load;
pub use load::*;

use bevy::prelude::*;

use crate::{prelude::EditorLoader, prefab::save::SaveState, EditorSet};


pub struct EditorCore;

impl Plugin for EditorCore {
    fn build(&self, app: &mut App) {
        app.add_event::<EditorEvent>();
        
        app.add_systems(Update, (apply_deferred, load_listener).chain().after(crate::prelude::bot_menu).in_set(EditorSet::Editor));
    }
}

#[derive(Event)]
pub enum EditorEvent {
    Load(String),
    Save(String)
}

fn editor_event_listener(
    mut events: EventReader<EditorEvent>,
    mut load_server : ResMut<EditorLoader>,
    assets : Res<AssetServer>,
    mut save_state : ResMut<NextState<SaveState>>,
) {
    for event in events.iter() {
        match event {
            EditorEvent::Load(path) => {
                    load_server.scene = Some(
                        assets.load(format!("{}.scn.ron", path))
                    );
            },
            EditorEvent::Save(path) => {
                save_state.set(SaveState::Save);
            },
        }
    }
}
