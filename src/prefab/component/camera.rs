use crate::ext::*;


//mark camera to run in play mode
#[derive(Component, Clone, Default, Reflect)]
#[reflect(Default, Component)]
pub struct CameraPlay {
    some_test_val : f32
}