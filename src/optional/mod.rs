use bevy::prelude::*;

#[cfg(feature = "bevy_xpbd_3d")]
pub mod bevy_xpbd_plugin;

#[cfg(feature = "f64")]
pub mod bevy_transform64_plugin;


//add optional dependencies
pub struct OptionalPlugin;

impl Plugin for OptionalPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "bevy_xpbd_3d")]
        {
            info!("Add bevy_xpbd_3d plugin to editor");
            app.add_plugins(bevy_xpbd_plugin::BevyXpbdPlugin);

        }
    }
}