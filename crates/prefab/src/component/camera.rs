use crate::ext::*;

/// Play Mode only camera tag/marker
#[cfg(not(tarpaulin_include))]
#[derive(Component, Clone, Default, Reflect)]
#[reflect(Default, Component)]
pub struct CameraPlay {}
