use bevy::prelude::*;

pub struct BevyTransform64Plugin;

impl Plugin for BevyTransform64Plugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_transform64::DTransformPlugin);
    }
}