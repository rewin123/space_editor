pub mod editor;
pub mod asset_insector;
pub mod prefab;
pub mod editor_registry;

use bevy::prelude::*;

use editor::EditorPlugin;
use editor_registry::EditorRegistryPlugin;
use prefab::PrefabPlugin;

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
        app.add_plugins(EditorRegistryPlugin);
        app.add_plugins(PrefabPlugin);
        app.add_plugins(EditorPlugin);
    }
}


//editor shows only entities with this marker
#[derive(Component)]
pub struct PrefabMarker;
