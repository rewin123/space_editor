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

#[derive(Reflect, Clone, InspectorOptions, Default, PartialEq, Eq, Debug)]
#[reflect(Default, InspectorOptions)]
pub struct AnimationClip {
    /// Animation clip first index in [`TextureAtlas`]
    pub first: usize,
    /// Animation clip last index in [`TextureAtlas`]
    pub last: usize,
}

#[derive(Component, Reflect, Clone, InspectorOptions, PartialEq, Eq, Debug)]
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

#[derive(Component, Reflect, Clone, InspectorOptions, PartialEq, Eq, Debug)]
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

#[derive(Component, Reflect, Clone, InspectorOptions, Deref, DerefMut, PartialEq, Eq, Debug)]
#[reflect(Default, Component, InspectorOptions)]
pub struct AnimationTimerSpriteSheet(Timer);

impl Default for AnimationTimerSpriteSheet {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, TimerMode::Repeating))
    }
}

#[derive(Component, Reflect, Clone, InspectorOptions, PartialEq, Debug)]
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
        texture_layout_assets: &mut Assets<TextureAtlasLayout>,
        asset_server: &AssetServer,
    ) -> Option<Handle<TextureAtlasLayout>> {
        let texture_handle = sprite_texture.to_texture(asset_server)?;
        self.texture = Some(texture_handle);

        let texture_layout = TextureAtlasLayout::from_grid(
            //self.tile_size,
            UVec2 {
                x: self.tile_size.x.round() as u32,
                y: self.tile_size.y.round() as u32,
            },
            self.columns as u32,
            self.rows as u32,
            Some(UVec2 {
                x: self.padding.unwrap_or_default().x.round() as u32,
                y: self.padding.unwrap_or_default().y.round() as u32,
            }),
            //self.offset,
            Some(UVec2 {
                x: self.offset.unwrap_or_default().x.round() as u32,
                y: self.offset.unwrap_or_default().y.round() as u32,
            }),
        );
        Some(texture_layout_assets.add(texture_layout))
    }
}

/// Function that manages the sprite animation execution
pub fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndicesSpriteSheet,
        &AnimationClipName,
        &mut AnimationTimerSpriteSheet,
        &mut TextureAtlas,
    )>,
) {
    for (sheet_indices, name, mut timer, mut atlas) in &mut query {
        timer.tick(time.delta());
        if timer.just_finished() {
            if let Some(indices) = sheet_indices.clips.get(&name.name) {
                atlas.index = if atlas.index == indices.last {
                    indices.first
                } else {
                    atlas.index + 1
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[test]
    fn sprite_texture_to_sprite_with_path() {
        let prefab = SpriteTexture {
            texture: String::from("test_asset.png"),
        };

        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default(),
        ));
        let server = app.world().resource::<AssetServer>();

        let sprite = prefab.to_sprite(server);

        assert!(sprite.is_some());
        let id = sprite.unwrap().texture.id();
        assert!(server.get_id_handle(id).is_some());
    }

    #[test]
    fn sprite_texture_to_sprite_with_fake_path() {
        let prefab = SpriteTexture {
            texture: String::from("fake_asset.png"),
        };

        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default(),
        ));
        let server = app.world().resource::<AssetServer>();

        let sprite = prefab.to_sprite(server);

        assert!(sprite.is_none());
    }

    #[test]
    fn spritesheet_texture_to_sprite_with_path() {
        let prefab = SpritesheetTexture {
            texture: String::from("test_asset.png"),
        };

        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default(),
        ));
        let server = app.world().resource::<AssetServer>();

        let sprite = prefab.to_texture(server);

        assert!(sprite.is_some());
        let id = sprite.unwrap().id();
        assert!(server.get_id_handle(id).is_some());
    }

    #[test]
    fn spritesheet_texture_to_sprite_with_fake_path() {
        let prefab = SpritesheetTexture {
            texture: String::from("fake_asset.png"),
        };

        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default(),
        ));
        let server = app.world().resource::<AssetServer>();

        let sprite = prefab.to_texture(server);

        assert!(sprite.is_none());
    }

    #[test]
    fn correct_default_animation_clips() {
        let animation_clips = AnimationIndicesSpriteSheet::default();

        assert_eq!(animation_clips.clips.len(), 2);
        assert_eq!(
            animation_clips.clips.get("run"),
            Some(&super::AnimationClip { first: 1, last: 6 })
        );
        assert_eq!(
            animation_clips.clips.get("idle"),
            Some(&super::AnimationClip { first: 0, last: 0 })
        );
    }

    #[test]
    fn default_available_animation_clips() {
        assert_eq!(
            AvailableAnimationClips::default().names,
            vec!["run".to_string(), "idle".to_string()],
        );
    }

    #[test]
    fn default_texture_atlas_to_texture_exists() {
        let sprite_prefab = SpritesheetTexture {
            texture: String::from("test_asset.png"),
        };
        let mut prefab = TextureAtlasPrefab::default();

        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default(),
        ))
        .init_asset::<TextureAtlasLayout>();

        let asset_server = app.world().resource::<AssetServer>().clone();
        let mut texture_atlas = app.world_mut().resource_mut::<Assets<TextureAtlasLayout>>();

        let sprite = prefab.to_texture_atlas(&sprite_prefab, &mut texture_atlas, &asset_server);

        assert!(sprite.is_some());
        let id = sprite.unwrap().id();
        assert!(texture_atlas.get(id).is_some());
    }

    #[test]
    fn default_animation_timer() {
        let anim_timer = AnimationTimerSpriteSheet::default();

        assert_eq!(anim_timer.0.mode(), TimerMode::Repeating);
        assert_eq!(anim_timer.0.duration(), Duration::from_secs_f32(0.1));
    }

    #[test]
    fn animate_sprite_over_time() {
        let setup = |mut commands: Commands| {
            commands.spawn((
                AnimationIndicesSpriteSheet::default(),
                AnimationClipName::default(),
                AnimationTimerSpriteSheet::default(),
                TextureAtlas::default(),
            ));
        };
        let mut app = App::new();

        app.add_plugins(MinimalPlugins)
            .add_systems(Startup, setup)
            .add_systems(Update, animate_sprite);

        app.update();
        let mut query = app.world_mut().query::<&TextureAtlas>();

        let atlas = query.single(&app.world());

        assert_eq!(atlas.index, 0);
    }
}
