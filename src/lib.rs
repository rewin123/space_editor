pub mod editor;
pub mod prefab;
pub mod editor_registry;

pub mod optional;

use bevy::prelude::*;

use editor::EditorPlugin;
use optional::OptionalPlugin;
use prefab::PrefabPlugin;

pub mod ext {
    pub use bevy_mod_picking::prelude::*;
    pub use bevy_inspector_egui::prelude::*;
    pub use bevy_egui::*;
}

pub mod prelude {
    pub use super::editor::prelude::*;
    pub use super::prefab::*;
    pub use super::SpaceEditorPlugin;
    pub use super::PrefabMarker;
    pub use super::editor_registry::*;
}

pub struct SpaceEditorPlugin {

}


impl Default for SpaceEditorPlugin {
    fn default() -> Self {
        Self {

        }
    }
}

impl Plugin for SpaceEditorPlugin {    
    fn build(&self, app: &mut App) {
        app.add_plugins(PrefabPlugin);
        app.add_plugins(OptionalPlugin);
        app.add_plugins(EditorPlugin);
    }
}


//editor shows only entities with this marker
#[derive(Component, Default, Clone, Reflect)]
#[reflect(Component)]
pub struct PrefabMarker;


#[derive(States, Default, Debug, Clone, Hash, Eq, PartialEq)]
pub enum EditorState {
    Editor, // editor is showing
    GamePrepare, //editor preparing to run game
    #[default]
    Game // playing game
}

#[derive(SystemSet, Hash, Eq, PartialEq, Clone, Debug)]
pub enum EditorSet {
    Editor,
    Game
}