#[cfg(feature = "bevy_xpbd_3d")]
use bevy::prelude::*;

#[cfg(feature = "bevy_xpbd_3d")]
pub mod bevy_xpbd_plugin;

//add optional dependencies
#[cfg(feature = "bevy_xpbd_3d")]
pub struct OptionalPlugin;

#[cfg(feature = "bevy_xpbd_3d")]
impl Plugin for OptionalPlugin {
    fn build(&self, app: &mut App) {
        {
            info!("Add bevy_xpbd_3d plugin to editor");
            app.add_plugins(bevy_xpbd_plugin::BevyXpbdPlugin);
        }
    }
}
