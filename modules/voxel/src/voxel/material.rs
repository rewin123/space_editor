use bevy::{
    log::info,
    prelude::{Color, Plugin, Resource},
    utils::HashMap,
};
use bitflags::bitflags;
use std::{any::type_name, any::TypeId};

use super::Voxel;

//todo: rewrite this in a way which allows constifying stuff.

// Registry info about a voxel material
#[derive(Default)]
pub struct MaterialRegistryInfo {
    pub name: &'static str,
    pub base_color: Color,
    pub flags: VoxelMaterialFlags,
    pub emissive: Color,
    pub perceptual_roughness: f32,
    pub metallic: f32,
    pub reflectance: f32,
}

/// Helper / marker trait for voxel materials.
pub trait VoxelMaterial {
    const ID: u8;

    fn into_voxel() -> Voxel {
        Voxel(Self::ID)
    }
}

#[macro_export]
macro_rules! voxel_material {
    ($types: ident, $id: expr) => {
        pub struct $types;
        impl $types {
            pub const NAME: &'static str = stringify!($types);
        }
        impl $crate::voxel::material::VoxelMaterial for $types {
            const ID: u8 = $id;
        }
    };
}

bitflags! {
    pub struct VoxelMaterialFlags : u32 {
        const SOLID = 0;
        const LIQUID = 1 << 1;
        const UNBREAKABLE = 1 << 2;
    }
}

impl Default for VoxelMaterialFlags {
    fn default() -> Self {
        Self::SOLID
    }
}

/// A registry for voxel material types.
/// This stores the voxel materials along their material id used to refer them in voxel data
#[derive(Resource)]
pub struct VoxelMaterialRegistry {
    materials: Vec<MaterialRegistryInfo>,
    mat_ids: HashMap<TypeId, usize>,
}

#[allow(dead_code)]
impl VoxelMaterialRegistry {
    #[inline]
    pub fn get_by_id(&self, id: u8) -> Option<&MaterialRegistryInfo> {
        self.materials.get(id as usize)
    }

    pub fn get_mut_by_id(&mut self, id: u8) -> Option<&mut MaterialRegistryInfo> {
        self.materials.get_mut(id as usize)
    }

    pub fn get_by_type<M: 'static>(&self) -> Option<&MaterialRegistryInfo> {
        self.mat_ids
            .get(&TypeId::of::<M>())
            .map(|x| self.materials.get(*x).unwrap())
    }

    pub fn get_id_for_type<M: 'static>(&self) -> Option<u8> {
        self.mat_ids.get(&TypeId::of::<M>()).map(|x| *x as u8)
    }

    pub fn register_material<M: 'static>(&mut self, mat: MaterialRegistryInfo) {
        self.materials.push(mat);
        info!(
            "Registered material {:?} (ID: {})",
            type_name::<M>(),
            self.materials.len() - 1
        );
        self.mat_ids.insert(TypeId::of::<M>(), self.materials.len());
    }

    pub fn iter_mats(&self) -> impl Iterator<Item = &MaterialRegistryInfo> {
        self.materials.iter()
    }
}

impl Default for VoxelMaterialRegistry {
    fn default() -> Self {
        let mut registry = Self {
            materials: Default::default(),
            mat_ids: Default::default(),
        };

        registry.register_material::<Void>(MaterialRegistryInfo {
            base_color: Color::BLACK,
            name: "Void",
            flags: VoxelMaterialFlags::SOLID,
            ..Default::default()
        });

        registry
    }
}

// The material with ID #0;
pub struct Void;

pub struct VoxelMaterialPlugin;
impl Plugin for VoxelMaterialPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.init_resource::<VoxelMaterialRegistry>();
    }
}
