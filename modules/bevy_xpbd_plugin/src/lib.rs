// Remove after update to newer rust version
#![allow(clippy::type_complexity)]
use bevy::prelude::*;

// Probably renaming this module would be a good idea
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
