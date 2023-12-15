//created just for loading together crates

#[allow(ambiguous_glob_reexports)]
pub mod prelude {
    pub use crate::SpaceEditorPlugin;
    #[cfg(feature = "bevy_xpbd_3d")]
    pub use bevy_xpbd_plugin::prelude::*;
    pub use editor::prelude::*;
    pub use editor::*;
    pub use prefab::prelude::*;
    pub use prefab::*;
    pub use shared::prelude::*;
    pub use shared::*;
}

#[cfg(feature = "bevy_xpbd_3d")]
pub use bevy_xpbd_plugin;

pub struct SpaceEditorPlugin;

impl bevy::app::Plugin for SpaceEditorPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_plugins(editor::EditorPlugin);

        #[cfg(feature = "bevy_xpbd_3d")]
        app.add_plugins(bevy_xpbd_plugin::XpbdPlugin);
    }
}
