use bevy::prelude::{Color, Plugin};

use crate::{
    voxel::material::{MaterialRegistryInfo, VoxelMaterialFlags, VoxelMaterialRegistry},
    voxel_material,
};

voxel_material!(Dirt, 1);
voxel_material!(Sand, 2);
voxel_material!(Grass, 3);
voxel_material!(Rock, 4);
voxel_material!(Snow, 5);
voxel_material!(Water, 6);
voxel_material!(Sandstone, 7);
voxel_material!(Bedrock, 8);
voxel_material!(Cactus, 9);
voxel_material!(Wood, 10);
voxel_material!(Leaves, 11);
voxel_material!(PineLeaves, 12);
voxel_material!(PineWood, 13);

pub struct VoxelWorldBaseMaterialsPlugin;

impl Plugin for VoxelWorldBaseMaterialsPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        let mut registry = app
            .world
            .get_resource_mut::<VoxelMaterialRegistry>()
            .unwrap();

        registry.register_material::<Dirt>(MaterialRegistryInfo {
            base_color: Color::rgb_u8(112, 97, 92),
            name: Dirt::NAME,
            flags: VoxelMaterialFlags::SOLID,
            emissive: Color::BLACK,
            perceptual_roughness: 0.75,
            reflectance: 0.45,
            ..Default::default()
        });

        registry.register_material::<Sand>(MaterialRegistryInfo {
            base_color: Color::rgb_u8(228, 219, 148),
            name: Sand::NAME,
            flags: VoxelMaterialFlags::SOLID,
            emissive: Color::BLACK,
            perceptual_roughness: 0.8,
            reflectance: 1.0,
            ..Default::default()
        });

        registry.register_material::<Grass>(MaterialRegistryInfo {
            base_color: Color::LIME_GREEN,
            name: Grass::NAME,
            flags: VoxelMaterialFlags::SOLID,
            emissive: Color::BLACK,
            perceptual_roughness: 0.66,
            reflectance: 0.3,
            ..Default::default()
        });

        registry.register_material::<Rock>(MaterialRegistryInfo {
            base_color: Color::GRAY,
            name: Rock::NAME,
            flags: VoxelMaterialFlags::SOLID,
            emissive: Color::BLACK,
            perceptual_roughness: 0.85,
            metallic: 0.6,
            ..Default::default()
        });

        registry.register_material::<Snow>(MaterialRegistryInfo {
            base_color: Color::WHITE,
            name: Snow::NAME,
            flags: VoxelMaterialFlags::SOLID,
            emissive: Color::BLACK,
            ..Default::default()
        });

        registry.register_material::<Water>(MaterialRegistryInfo {
            base_color: *Color::rgb_u8(78, 167, 215).set_a(0.4),
            name: Water::NAME,
            flags: VoxelMaterialFlags::LIQUID,
            emissive: Color::BLACK,
            perceptual_roughness: 0.2,
            metallic: 0.47,
            ..Default::default()
        });

        registry.register_material::<Sandstone>(MaterialRegistryInfo {
            base_color: Color::rgb_u8(198, 192, 144),
            name: Sandstone::NAME,
            flags: VoxelMaterialFlags::SOLID,
            emissive: Color::BLACK,
            ..Default::default()
        });

        registry.register_material::<Bedrock>(MaterialRegistryInfo {
            base_color: Color::DARK_GRAY,
            name: Bedrock::NAME,
            flags: VoxelMaterialFlags::UNBREAKABLE,
            emissive: Color::BLACK,
            perceptual_roughness: 0.9,
            metallic: 1.0,
            ..Default::default()
        });

        registry.register_material::<Cactus>(MaterialRegistryInfo {
            base_color: Color::rgb_u8(0, 96, 0),
            name: Cactus::NAME,
            flags: VoxelMaterialFlags::SOLID,
            emissive: Color::BLACK,
            ..Default::default()
        });

        registry.register_material::<Wood>(MaterialRegistryInfo {
            base_color: Color::rgb_u8(188, 147, 97),
            name: Wood::NAME,
            flags: VoxelMaterialFlags::SOLID,
            emissive: Color::BLACK,
            perceptual_roughness: 0.7,
            metallic: 0.46,
            ..Default::default()
        });

        registry.register_material::<Leaves>(MaterialRegistryInfo {
            base_color: Color::rgb_u8(109, 177, 56),
            name: Leaves::NAME,
            flags: VoxelMaterialFlags::SOLID,
            emissive: Color::BLACK,
            perceptual_roughness: 0.73,
            metallic: 1.0,
            ..Default::default()
        });

        registry.register_material::<PineLeaves>(MaterialRegistryInfo {
            base_color: Color::rgb_u8(135, 201, 167),
            name: PineLeaves::NAME,
            flags: VoxelMaterialFlags::SOLID,
            emissive: Color::BLACK,
            perceptual_roughness: 0.73,
            metallic: 1.0,
            ..Default::default()
        });

        registry.register_material::<PineWood>(MaterialRegistryInfo {
            base_color: Color::rgb_u8(174, 155, 126),
            name: PineWood::NAME,
            flags: VoxelMaterialFlags::SOLID,
            emissive: Color::BLACK,
            perceptual_roughness: 0.7,
            metallic: 0.46,
            ..Default::default()
        });
    }
}
