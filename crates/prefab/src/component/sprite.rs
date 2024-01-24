use crate::ext::*;
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
