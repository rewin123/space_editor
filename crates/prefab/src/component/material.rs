use crate::ext::*;

/// Prefab component that store parameters and asset paths for creating [`StandardMaterial`]
#[derive(Component, Reflect, Clone)]
#[reflect(Default, Component)]
pub struct MaterialPrefab {
    pub base_color: Color,
    pub base_color_texture: String,
    pub emissive: Color,
    pub emissive_texture: String,
    pub perceptual_roughness: f32,
    pub metallic: f32,
    pub metallic_roughness_texture: String,
    pub reflectance: f32,
    pub normal_map_texture: String,
    pub flip_normal_map_y: bool,
    pub occlusion_texture: String,
    pub double_sided: bool,
    pub unlit: bool,
    pub fog_enabled: bool,
    pub alpha_mode: AlphaMode,
    pub depth_bias: f32,
    pub depth_map: String,
    pub parallax_depth_scale: f32,
    pub parallax_mapping_method: ParallaxMappingMethod,
    pub max_parallax_layer_count: f32,
}

impl Default for MaterialPrefab {
    fn default() -> Self {
        Self {
            base_color: Color::rgb(1.0, 1.0, 1.0),
            base_color_texture: String::default(),
            emissive: Color::BLACK,
            emissive_texture: String::default(),
            // Matches Blender's default roughness.
            perceptual_roughness: 0.5,
            // Metallic should generally be set to 0.0 or 1.0.
            metallic: 0.0,
            metallic_roughness_texture: String::default(),
            // Minimum real-world reflectance is 2%, most materials between 2-5%
            // Expressed in a linear scale and equivalent to 4% reflectance see
            // <https://google.github.io/filament/Material%20Properties.pdf>
            reflectance: 0.5,
            occlusion_texture: String::default(),
            normal_map_texture: String::default(),
            flip_normal_map_y: false,
            double_sided: false,
            unlit: false,
            fog_enabled: true,
            alpha_mode: AlphaMode::Opaque,
            depth_bias: 0.0,
            depth_map: String::default(),
            parallax_depth_scale: 0.1,
            max_parallax_layer_count: 16.0,
            parallax_mapping_method: ParallaxMappingMethod::Occlusion,
        }
    }
}

fn try_image(path: &String, asset_server: &AssetServer) -> Option<Handle<Image>> {
    if path.is_empty() {
        None
    } else {
        Some(asset_server.load(path))
    }
}

impl MaterialPrefab {
    /// Convert [`MaterialPrefab`] to [`StandardMaterial`]
    pub fn to_material(&self, asset_server: &AssetServer) -> StandardMaterial {
        let base_color_texture = try_image(&self.base_color_texture, asset_server);
        let emissive_texture = try_image(&self.emissive_texture, asset_server);
        let metallic_roughness_texture = try_image(&self.metallic_roughness_texture, asset_server);
        let normal_map_texture = try_image(&self.normal_map_texture, asset_server);
        let occlusion_texture = try_image(&self.occlusion_texture, asset_server);
        let depth_map = try_image(&self.depth_map, asset_server);
        StandardMaterial {
            base_color: self.base_color,
            base_color_texture,
            emissive: self.emissive,
            emissive_texture,
            perceptual_roughness: self.perceptual_roughness,
            metallic: self.metallic,
            metallic_roughness_texture,
            reflectance: self.reflectance,
            normal_map_texture,
            flip_normal_map_y: self.flip_normal_map_y,
            occlusion_texture,
            double_sided: self.double_sided,
            unlit: self.unlit,
            fog_enabled: self.fog_enabled,
            alpha_mode: self.alpha_mode,
            depth_bias: self.depth_bias,
            depth_map,
            parallax_depth_scale: self.parallax_depth_scale,
            parallax_mapping_method: self.parallax_mapping_method,
            max_parallax_layer_count: self.max_parallax_layer_count,
            ..Default::default()
        }
    }
}
