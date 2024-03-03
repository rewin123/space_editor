use crate::voxel::material::VoxelMaterialRegistry;
use bevy::{
    prelude::*,
    reflect::TypePath,
    render::{
        extract_component::ExtractComponent,
        mesh::MeshVertexAttribute,
        render_resource::{AsBindGroup, ShaderType, VertexFormat},
    },
};

#[derive(Component, Clone, Default, ExtractComponent)]
/// A marker component for voxel meshes.
pub struct VoxelTerrainMesh;

impl VoxelTerrainMesh {
    pub const ATTRIBUTE_DATA: MeshVertexAttribute =
        MeshVertexAttribute::new("Vertex_Data", 0x696969, VertexFormat::Uint32);
}

#[derive(ShaderType, Clone, Copy, Debug, Default)]
pub struct GpuVoxelMaterial {
    base_color: Color,
    flags: u32,
    emissive: Color,
    perceptual_roughness: f32,
    metallic: f32,
    reflectance: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GpuTerrainUniforms {
    #[uniform(0)]
    pub render_distance: u32,
    #[uniform(1)]
    pub materials: [GpuVoxelMaterial; 256],
}

impl Default for GpuTerrainUniforms {
    fn default() -> Self {
        Self {
            render_distance: 16,
            materials: [default(); 256],
        }
    }
}

impl Material for GpuTerrainUniforms {
    fn vertex_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/terrain_pipeline.wgsl".into()
    }

    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "shaders/terrain_pipeline.wgsl".into()
    }

    fn specialize(
        _pipeline: &bevy::pbr::MaterialPipeline<Self>,
        descriptor: &mut bevy::render::render_resource::RenderPipelineDescriptor,
        layout: &bevy::render::mesh::MeshVertexBufferLayout,
        _key: bevy::pbr::MaterialPipelineKey<Self>,
    ) -> Result<(), bevy::render::render_resource::SpecializedMeshPipelineError> {
        let vertex_layout = layout.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            VoxelTerrainMesh::ATTRIBUTE_DATA.at_shader_location(1),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

fn update_chunk_material_singleton(
    mut commands: Commands,
    mut materials: ResMut<Assets<GpuTerrainUniforms>>,
    chunk_material: ResMut<ChunkMaterialSingleton>,
    voxel_materials: Res<VoxelMaterialRegistry>,
    mut chunk_entities: Query<(Entity, &mut Handle<GpuTerrainUniforms>)>,
) {
    if chunk_material.is_changed() {
        let mut gpu_mats = GpuTerrainUniforms {
            materials: [GpuVoxelMaterial {
                base_color: Color::WHITE,
                flags: 0,
                ..Default::default()
            }; 256],
            render_distance: 32,
        };

        voxel_materials
            .iter_mats()
            .enumerate()
            .for_each(|(index, material)| {
                gpu_mats.materials[index].base_color = material.base_color;
                gpu_mats.materials[index].flags = material.flags.bits();
                gpu_mats.materials[index].emissive = material.emissive;
                gpu_mats.materials[index].perceptual_roughness = material.perceptual_roughness;
                gpu_mats.materials[index].metallic = material.metallic;
                gpu_mats.materials[index].reflectance = material.reflectance;
            });

        let chunk_material = materials.add(gpu_mats);
        commands.insert_resource(ChunkMaterialSingleton(chunk_material.clone()));

        for (_, mut mat) in &mut chunk_entities {
            *mat = chunk_material.clone();
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct ChunkMaterialSingleton(Handle<GpuTerrainUniforms>);

impl FromWorld for ChunkMaterialSingleton {
    fn from_world(world: &mut World) -> Self {
        let mut materials = world.resource_mut::<Assets<GpuTerrainUniforms>>();
        Self(materials.add(GpuTerrainUniforms::default()))
    }
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq, SystemSet)]
/// Systems that prepare the global [ChunkMaterialSingleton] value.
pub struct ChunkMaterialSet;

pub struct ChunkMaterialPlugin;

impl Plugin for ChunkMaterialPlugin {
    fn build(&self, app: &mut App) {
        // @todo: figure out race conditions w/ other systems
        app.add_plugins(MaterialPlugin::<GpuTerrainUniforms>::default())
            .init_resource::<ChunkMaterialSingleton>()
            .add_systems(
                Update,
                update_chunk_material_singleton
                    .run_if(resource_changed::<VoxelMaterialRegistry>())
                    .in_set(ChunkMaterialSet),
            );
    }
}
