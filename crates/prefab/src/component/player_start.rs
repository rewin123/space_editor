use crate::ext::*;

/// Entities with this component will spawn prefab on enter to [`EditorState::Game`] state
///
/// [`EditorState::Game`]: crate::EditorState::Game
#[derive(Component, Reflect, Clone)]
#[reflect(Component, Default)]
pub struct PlayerStart {
    pub prefab: String,
}

impl Default for PlayerStart {
    fn default() -> Self {
        Self {
            prefab: "".to_string(),
        }
    }
}
