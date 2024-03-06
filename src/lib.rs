//created just for loading together crates

/// Contains all the functions/structs/plugins of kenney_city_editor
pub mod prelude {
    pub use crate::SpaceEditorPlugin;
    pub use kcg_editor_ui::prelude::*;

    #[cfg(feature = "bevy_xpbd_3d")]
    pub use kcg_bevy_xpbd_plugin::prelude::*;
}

pub use kcg_editor_ui;
pub use kcg_prefab;

#[cfg(feature = "bevy_xpbd_3d")]
pub use kcg_bevy_xpbd_plugin;

/// This is the main plugin, connecting it will allow you to use all the functions of kenney_city_editor
pub struct SpaceEditorPlugin;

impl bevy::app::Plugin for SpaceEditorPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(kcg_editor_ui::EditorPlugin);

        #[cfg(feature = "bevy_xpbd_3d")]
        app.add_plugins(kcg_bevy_xpbd_plugin::XpbdPlugin);
    }
}
