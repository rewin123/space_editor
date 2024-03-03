use crate::ext::*;

/// Entities with this component will spawn prefab on enter to [`EditorState::Game`] state
///
/// [`EditorState::Game`]: crate::EditorState::Game
#[cfg(not(tarpaulin_include))]
#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default)]
pub struct PlayerStart {
    pub prefab: String,
}

#[cfg_attr(tarpaulin, ignore)]
impl Default for PlayerStart {
    fn default() -> Self {
        Self {
            prefab: "".to_string(),
        }
    }
}
