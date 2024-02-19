

// need to finish me !!! 
// see https://github.com/ethereumdegen/bevy_terrain/blob/main/assets/shaders/advanced.wgsl 


#import bevy_pbr::mesh_vertex_output MeshVertexOutput

struct CustomMaterial {
    color: vec4<f32>,
};

@group(1) @binding(0)
var<uniform> material: CustomMaterial;
@group(1) @binding(1)
var base_color_texture: texture_2d_array<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;
 

//use a U16 bit texture for splat mapping where each pixel acts like a bitfield..?    


@fragment
fn fragment(
    mesh: MeshVertexOutput,
) -> @location(0) vec4<f32> {
    
    //ultimately need to load this from some sort of binding -- like a texture sample.  
    let texture_splat_index = 0; 
    
    //also need to make sure the mesh UVs make sense ... 
    
    return material.color * textureSample(base_color_texture, base_color_sampler, mesh.uv , texture_splat_index);
}