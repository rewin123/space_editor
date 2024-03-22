use crate::ext::*;

/// Play Mode only camera tag/marker
#[cfg(not(tarpaulin_include))]
#[derive(Component, Clone, Default, Reflect)]
#[reflect(Default, Component)]
pub struct PlaymodeCamera {}

/// Play Mode only Light tag/marker
#[cfg(not(tarpaulin_include))]
#[derive(Component, Clone, Default, Reflect)]
#[reflect(Default, Component)]
pub struct PlaymodeLight {}
