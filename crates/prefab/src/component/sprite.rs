use crate::ext::*;
use bevy::utils::HashMap;
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};

use super::material::try_image;

/// Prefab component that store parameters and asset paths for creating [`StandardMaterial`]
#[derive(Component, Reflect, Clone, InspectorOptions, Default)]
#[reflect(Default, Component, InspectorOptions)]
pub struct SpriteTexture {
    pub texture: String,
}

impl SpriteTexture {
    /// Convert [`SpriteTexture`] to [`SpriteBundle`]
    pub fn to_sprite(&self, asset_server: &AssetServer) -> Option<SpriteBundle> {
        let texture = try_image(&self.texture, asset_server)?;
        Some(SpriteBundle {
            texture,
            ..default()
        })
    }
}

/// Prefab component that store parameters and asset paths for creating [`StandardMaterial`]
#[derive(Component, Reflect, Clone, InspectorOptions, Default)]
#[reflect(Default, Component, InspectorOptions)]
pub struct SpritesheetTexture {
    pub texture: String,
}

impl SpritesheetTexture {
    /// Convert [`SpritesheetTexture`] to [`TextureHandle`]
    pub fn to_texture(&self, asset_server: &AssetServer) -> Option<Handle<Image>> {
        try_image(&self.texture, asset_server)
    }
}

#[derive(Component, Reflect, Clone, InspectorOptions)]
#[reflect(Default, Component, InspectorOptions)]
pub struct AnimationIndicesSpriteSheet {
    /// Collection of clips and their index ranges in the Texture Atlas
    pub clips: HashMap<String, AnimationClip>,
}

impl Default for AnimationIndicesSpriteSheet {
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert(String::from("run"), AnimationClip { first: 1, last: 6 });
        map.insert(String::from("idle"), AnimationClip { first: 0, last: 0 });
        Self { clips: map }
    }
}

#[derive(Reflect, Clone, InspectorOptions, Default)]
#[reflect(Default, InspectorOptions)]
pub struct AnimationClip {
    /// Animation clip first index in [`TextureAtlas`]
    pub first: usize,
    /// Animation clip last index in [`TextureAtlas`]
    pub last: usize,
}

#[derive(Component, Reflect, Clone, InspectorOptions)]
#[reflect(Default, Component, InspectorOptions)]
pub struct AnimationClipName {
    /// Animation clip name to play in [`TextureAtlas`]
    pub name: String,
}

impl Default for AnimationClipName {
    fn default() -> Self {
        Self {
            name: "run".to_string(),
        }
    }
}

#[derive(Component, Reflect, Clone, InspectorOptions)]
#[reflect(Default, Component, InspectorOptions)]
pub struct AvailableAnimationClips {
    /// List of all available animation clips by name to play in [`TextureAtlas`]
    pub names: Vec<String>,
}

impl Default for AvailableAnimationClips {
    fn default() -> Self {
        Self {
            names: vec!["run".to_string(), "idle".to_string()],
        }
    }
}

#[derive(Component, Reflect, Clone, InspectorOptions, Deref, DerefMut)]
#[reflect(Default, Component, InspectorOptions)]
pub struct AnimationTimerSpriteSheet(Timer);

impl Default for AnimationTimerSpriteSheet {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, TimerMode::Repeating))
    }
}

#[derive(Component, Reflect, Clone, InspectorOptions)]
#[reflect(Default, Component, InspectorOptions)]
pub struct TextureAtlasPrefab {
    /// Source texture
    pub texture: Option<Handle<Image>>,
    /// Size of the tile in the sprite sheet texture
    pub tile_size: Vec2,
    /// Number of columns in the texture
    pub columns: usize,
    /// Number of rows in the texture
    pub rows: usize,
    /// Texture padding in the sprite sheet
    pub padding: Option<Vec2>,
    /// Texture offset in the sprite sheet
    pub offset: Option<Vec2>,
}

impl Default for TextureAtlasPrefab {
    fn default() -> Self {
        Self {
            texture: None,
            tile_size: Vec2::new(24.0, 24.0),
            columns: 7,
            rows: 1,
            padding: None,
            offset: None,
        }
    }
}

impl TextureAtlasPrefab {
    pub fn to_texture_atlas(
        &mut self,
        sprite_texture: &SpritesheetTexture,
        texture_atlases: &mut ResMut<Assets<TextureAtlas>>,
        asset_server: &AssetServer,
    ) -> Option<Handle<TextureAtlas>> {
        let texture_handle = sprite_texture.to_texture(asset_server)?;
        self.texture = Some(texture_handle.clone());

        let texture_atlas = TextureAtlas::from_grid(
            texture_handle,
            self.tile_size,
            self.columns,
            self.rows,
            self.padding,
            self.offset,
        );
        Some(texture_atlases.add(texture_atlas))
    }
}

/// Function that manages the sprite animation execution
pub fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndicesSpriteSheet,
        &AnimationClipName,
        &mut AnimationTimerSpriteSheet,
        &mut TextureAtlasSprite,
    )>,
) {
    for (sheet_indices, name, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            if let Some(indices) = sheet_indices.clips.get(&name.name) {
                sprite.index = if sprite.index == indices.last {
                    indices.first
                } else {
                    sprite.index + 1
                };
            }
        }
    }
}
