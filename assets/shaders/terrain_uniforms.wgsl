const VOXEL_MAT_FLAG_LIQUID: u32 = 2u; // 1 << 1
const TERRAIN_CHUNK_LENGTH: u32 = 32u;

struct VoxelMat {
    base_color: vec4<f32>,
    flags: u32,
    emissive: vec4<f32>,
    perceptual_roughness: f32,
    metallic: f32,
    reflectance: f32,
};

@group(1) @binding(0)
var<uniform> render_distance: u32;

// A GPU-suited representation of voxel materials.
@group(1) @binding(1)
var<uniform> voxel_materials: array<VoxelMat, 256>;