use crate::ext::*;
use bevy::utils::HashMap;
use bevy_inspector_egui::{inspector_options::ReflectInspectorOptions, InspectorOptions};

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

fn try_image(path: &String, asset_server: &AssetServer) -> Option<Handle<Image>> {
    if path.is_empty() {
        None
    } else {
        Some(asset_server.load(path))
    }
}

// Spritesheet

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
    clips: HashMap<String, AnimationClip>,
}

impl Default for AnimationIndicesSpriteSheet {
    fn default() -> Self {
        let mut map = HashMap::new();
        map.insert(String::from("run"), AnimationClip { first: 1, last: 6 });
        Self { clips: map }
    }
}

#[derive(Reflect, Clone, InspectorOptions, Default)]
#[reflect(Default, InspectorOptions)]
pub struct AnimationClip {
    /// Animation clip first index in [`TextureAtlas`]
    first: usize,
    /// Animation clip last index in [`TextureAtlas`]
    last: usize,
}

#[derive(Component, Reflect, Clone, InspectorOptions, Deref, DerefMut)]
#[reflect(Default, Component, InspectorOptions)]
pub struct AnimationTimerSpriteSheet(Timer);

impl Default for AnimationTimerSpriteSheet {
    fn default() -> Self {
        Self(Timer::from_seconds(0.1, TimerMode::Repeating))
    }
}

#[derive(Component, Reflect, Clone, InspectorOptions, Default)]
#[reflect(Default, Component, InspectorOptions)]
pub struct TextureAtlasPrefab {
    /// Source texture
    texture: Handle<Image>,
    /// Size of the tile in the sprite sheet texture
    tile_size: Vec2,
    /// Number of columns in the texture
    columns: usize,
    /// Number of rows in the texture
    rows: usize,
    /// Texture padding in the sprite sheet
    padding: Option<Vec2>,
    /// Texture offset in the sprite sheet
    offset: Option<Vec2>,
}

impl TextureAtlasPrefab {
    pub fn to_texture_atlas(
        &self,
        sprite_texture: SpritesheetTexture,
        mut texture_atlases: ResMut<Assets<TextureAtlas>>,
        asset_server: &AssetServer,
    ) -> Option<Handle<TextureAtlas>> {
        let texture_handle = sprite_texture.to_texture(asset_server)?;
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

#[derive(Component, Reflect, Clone, InspectorOptions, Default)]
#[reflect(Default, Component, InspectorOptions)]
pub struct TextureAtlasSpritePrefab {
    /// The tint color used to draw the sprite, defaulting to [`Color::WHITE`]
    pub color: Color,
    /// Texture index in [`TextureAtlas`]
    pub index: usize,
    /// Whether to flip the sprite in the X axis
    pub flip_x: bool,
    /// Whether to flip the sprite in the Y axis
    pub flip_y: bool,
    /// An optional custom size for the sprite that will be used when rendering, instead of the size
    /// of the sprite's image in the atlas
    pub custom_size: Option<Vec2>,
    /// [`Anchor`] point of the sprite in the world
    pub anchor: Anchor,
}

/// System to sync [`SpriteBundle`] and [`SpriteTexture`]
pub fn sync_spritesheet(
    mut commands: Commands,
    query: Query<
        (Entity, &SpritesheetTexture, &AnimationIndicesSpriteSheet, &TextureAtlasPrefab, &TextureAtlasSpritePrefab),
        Or<(
            Changed<SpritesheetTexture>,
            Changed<AnimationIndicesSpriteSheet>,
            Changed<TextureAtlasPrefab>,
            Changed<TextureAtlasSpritePrefab>,
        )>,
    >,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    for (e, prefab) in query.iter() {
        if let Some(sprite) = prefab.to_sprite(&asset_server) {
            commands.entity(e).insert(sprite);
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("textures/rpg/chars/gabe/gabe-idle-run.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndicesSpriteSheet::default();
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(animation_indices.first),
            transform: Transform::from_scale(Vec3::splat(6.0)),
            ..default()
        },
        animation_indices,
        AnimationTimerSpriteSheet(Timer::from_seconds(0.1, TimerMode::Repeating)),
    ));
}
