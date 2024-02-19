//created just for loading together crates

/// Contains all the functions/structs/plugins of space_editor
pub mod prelude {
    pub use crate::SpaceEditorPlugin;
    pub use space_editor_ui::prelude::*;

    #[cfg(feature = "bevy_xpbd_3d")]
    pub use space_bevy_xpbd_plugin::prelude::*;
}

pub use space_editor_ui;
pub use space_prefab;

#[cfg(feature = "bevy_xpbd_3d")]
pub use space_bevy_xpbd_plugin;

/// This is the main plugin, connecting it will allow you to use all the functions of space_editor
pub struct SpaceEditorPlugin;

impl bevy::app::Plugin for SpaceEditorPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(space_editor_ui::EditorPlugin);

        #[cfg(feature = "bevy_xpbd_3d")]
        app.add_plugins(space_bevy_xpbd_plugin::XpbdPlugin);

        #[cfg(feature = "bevy_mesh_terrain")]
        app.add_plugins(bevy_mesh_terrain_plugin::BevyMeshTerrainPlugin);
    }
}
