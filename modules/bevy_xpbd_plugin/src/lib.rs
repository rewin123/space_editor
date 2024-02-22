// Remove after update to newer rust version
#![allow(clippy::type_complexity)]
use bevy::prelude::*;

pub mod collider;
pub mod registry;
pub mod spatial_query;

/// Community module containing bevy_xpbd_3d plugin
pub struct XpbdPlugin;

impl Plugin for XpbdPlugin {
    fn build(&self, app: &mut App) {
        {
            info!("Add bevy_xpbd_3d plugin to editor");
            app.add_plugins(registry::BevyXpbdPlugin);
            app.register_type::<Option<Vec3>>();
            app.register_type::<Option<Color>>();
            app.register_type::<Option<[f32; 4]>>();
            app.register_type::<[f32; 4]>();
        }
    }
}

pub mod prelude {
    pub use crate::collider::*;
    pub use crate::registry::*;
    pub use crate::spatial_query::*;
    pub use crate::XpbdPlugin;
    pub use bevy_xpbd_3d;
}
