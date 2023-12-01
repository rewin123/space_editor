use bevy::prelude::*;

pub mod bevy_xpbd_plugin;

//add optional dependencies
pub struct XpbdPlugin;

impl Plugin for XpbdPlugin {
    fn build(&self, app: &mut App) {
        {
            info!("Add bevy_xpbd_3d plugin to editor");
            app.add_plugins(bevy_xpbd_plugin::BevyXpbdPlugin);
        }
    }
}
