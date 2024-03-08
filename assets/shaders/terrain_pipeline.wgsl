#import bevy_pbr::{
    clustered_forward,
    fog,
    lighting,
    mesh_bindings::mesh,
    mesh_view_bindings::view,
    mesh_functions,
    pbr_bindings,
    pbr_functions::{calculate_view, apply_pbr_lighting},
    pbr_types::{PbrInput, pbr_input_new},
    shadows,
    utils,
    view_transformations
}
#import bevy_core_pipeline::tonemapping::tone_mapping

#import "shaders/voxel_data.wgsl"::{voxel_data_extract_normal, voxel_data_extract_material_index}
#import "shaders/terrain_uniforms.wgsl"::{VoxelMat, voxel_materials, render_distance, TERRAIN_CHUNK_LENGTH}
#import "shaders/noise.wgsl"::hash
#import "shaders/fog.wgsl"::ffog_apply_fog

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) voxel_data: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) voxel_normal: vec3<f32>,
    @location(1) voxel_data: u32,
    @location(2) world_position: vec3<f32>,
    @location(3) instance_index: u32,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let model = mesh_functions::get_model_matrix(vertex.instance_index);
    let world_position = bevy_pbr::mesh_functions::mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));

    var out: VertexOutput;
    let voxel_normal = voxel_data_extract_normal(vertex.voxel_data);
    out.clip_position = view_transformations::position_world_to_clip(world_position.xyz);
    out.voxel_normal = voxel_normal;
    out.voxel_data = vertex.voxel_data;
    out.world_position = world_position.xyz;
    out.instance_index = vertex.instance_index;

    return out;
}

struct Fragment {
    @builtin(position) frag_coord: vec4<f32>,
    @builtin(front_facing) front_facing: bool,
    /// The normalized normal of the voxel.
    @location(0) voxel_normal: vec3<f32>,
    /// The voxel data.
    @location(1) voxel_data: u32,
    /// The world position of the voxel vertex.
    @location(2) world_position: vec3<f32>,
    @location(3) instance_index: u32,
};

fn prepare_pbr_input_from_voxel_mat(voxel_mat: VoxelMat, frag: Fragment) -> PbrInput {
    var base_color: vec4<f32> = voxel_mat.base_color;
    base_color = base_color + hash(vec4<f32>(floor(frag.world_position - frag.voxel_normal * 0.5), 1.0)) * 0.0226;

    let voxel_world_normal = bevy_pbr::mesh_functions::mesh_normal_local_to_world(frag.voxel_normal, frag.instance_index);

    var pbr_input: PbrInput = pbr_input_new();
    pbr_input.material.metallic = voxel_mat.metallic;
    pbr_input.material.perceptual_roughness = voxel_mat.perceptual_roughness;
    pbr_input.material.emissive = voxel_mat.emissive;
    pbr_input.material.reflectance = voxel_mat.reflectance;
    pbr_input.material.base_color = base_color;

    pbr_input.frag_coord = frag.frag_coord;
    pbr_input.world_position = vec4<f32>(frag.world_position, 1.0);
    pbr_input.world_normal = (f32(frag.front_facing) * 2.0 - 1.0) * voxel_world_normal;

    pbr_input.is_orthographic = view.projection[3].w == 1.0;
    pbr_input.N = normalize(voxel_world_normal);
    pbr_input.V = calculate_view(vec4<f32>(frag.world_position, 1.0), pbr_input.is_orthographic);
    pbr_input.flags = mesh[frag.instance_index].flags;

    return pbr_input;
}

@fragment
fn fragment(frag: Fragment) -> @location(0) vec4<f32> {
    let material = voxel_materials[voxel_data_extract_material_index(frag.voxel_data)];

    /// PBR lighting input data preparation
    var pbr_input = prepare_pbr_input_from_voxel_mat(material, frag);
    let pbr_colour = tone_mapping(apply_pbr_lighting(pbr_input), view.color_grading);

    // @todo: switch to bevy_pbr::fog

    //fragment distance from camera, used to determine amount of fog to apply.
    let fog_distance = distance(frag.world_position, view.world_position);
    return ffog_apply_fog(fog_distance, f32(render_distance) * f32(TERRAIN_CHUNK_LENGTH), f32(TERRAIN_CHUNK_LENGTH), pbr_colour);
}