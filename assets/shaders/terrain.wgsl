 
//see bindings in terrain_material.rs 
 
 
 #import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::alpha_discard,
    pbr_fragment::pbr_input_from_standard_material,
      pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
    pbr_types::STANDARD_MATERIAL_FLAGS_UNLIT_BIT,
}
  
struct StandardMaterial {
    time: f32,
    base_color: vec4<f32>,
    emissive: vec4<f32>,
    perceptual_roughness: f32,
    metallic: f32,
    reflectance: f32,
    // 'flags' is a bit field indicating various options. u32 is 32 bits so we have up to 32 options.
    flags: u32,
    alpha_cutoff: f32,
};


struct ChunkMaterialUniforms {
    color_texture_expansion_factor: f32 ,
    chunk_uv: vec4<f32>,  //start_x, start_y, end_x, end_y   -- used to subselect a region from the splat texture 
    
};

//https://github.com/DGriffin91/bevy_mod_standard_material/blob/main/assets/shaders/pbr.wgsl


@group(1) @binding(1)
var base_color_texture1: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler1: sampler;
 

@group(1) @binding(3)
var emissive_texture: texture_2d<f32>;
@group(1) @binding(4)
var emissive_sampler: sampler;

@group(1) @binding(5)
var metallic_roughness_texture: texture_2d<f32>;
@group(1) @binding(6)
var metallic_roughness_sampler: sampler;

@group(1) @binding(7)
var occlusion_texture: texture_2d<f32>;
@group(1) @binding(8)
var occlusion_sampler: sampler;


@group(1) @binding(20)
var<uniform> chunk_uniforms: ChunkMaterialUniforms;
@group(1) @binding(21)
var base_color_texture: texture_2d_array<f32>;
@group(1) @binding(22)
var base_color_sampler: sampler;


//the splat map texture has 3 channels: R, G, B
//R tells us the terrain_layer_index 0 per pixel
//G tells us the terrain_layer_index 1 per pixel
//B is 0-255 mapped to 0 to 100% telling us how much of R to render versus how much of G to render 
@group(1) @binding(23)
 var splat_map_texture: texture_2d<f32>; 
//var splat_map_texture: texture_2d_array<f32>; //these are control maps and there will be 4 
@group(1) @binding(24)
var splat_map_sampler: sampler;

//works similar to splat mask  -- we use a separate tex for this for NOW to make collision mesh building far easier (only need height map and not splat)
@group(1) @binding(25)
var alpha_mask_texture: texture_2d<f32>; 
@group(1) @binding(26)
var alpha_mask_sampler: sampler;
 


//should consider adding vertex painting to this .. need another binding of course.. performs a color shift 

@fragment
fn fragment(
    mesh: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> @location(0) vec4<f32> {
    
   
   // let tiled_uv = material.color_texture_expansion_factor*mesh.uv;  //cannot get this binding to work !? 
    let tiled_uv = 8.0*mesh.uv;
    
    
    // seems to be working !! yay ! makes our splat texture encompass all of the chunks 
    let splat_uv = chunk_uniforms.chunk_uv.xy + mesh.uv * (chunk_uniforms.chunk_uv.zw - chunk_uniforms.chunk_uv.xy);
    
    let splat_values = textureSample(splat_map_texture, splat_map_sampler, splat_uv );
    let alpha_mask_value = textureSample(alpha_mask_texture, alpha_mask_sampler, splat_uv );  //comes from height map atm but COULD come from splat map now 
    
       //comes from the  control map .. float -> integer 
    let terrain_layer_index_0 = i32( splat_values.r * 255.0 );     ///* 255.0
    let terrain_layer_index_1 = i32( splat_values.g * 255.0 );
    
    //this technique lets us use 255 total textures BUT we can only layer 2 at a time.  
    let color_from_texture_0 = textureSample(base_color_texture, base_color_sampler, tiled_uv, terrain_layer_index_0);
    let color_from_texture_1 = textureSample(base_color_texture, base_color_sampler, tiled_uv, terrain_layer_index_1);
    

    let blend_amount = splat_values.b;  //comes from B channel -- this pixel 
      
    

    let blended_color = color_from_texture_0 * (1.0 - blend_amount) +
                        color_from_texture_1 * (blend_amount)  ;


   
  // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(mesh, is_front);
 
    //hack the material (StandardMaterialUniform)  so the color is from the terrain splat 
    pbr_input.material.base_color =  blended_color;


    var pbr_out: FragmentOutput;
 
    
    // apply lighting
    pbr_out.color = apply_pbr_lighting(pbr_input);
    // we can optionally modify the lit color before post-processing is applied
    // out.color = out.color;
    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    pbr_out.color = main_pass_post_lighting_processing(pbr_input, pbr_out.color);



    // -----

   // let shadowFactor = calculate_shadow_factor(frag_lightSpacePos);

    let final_color = vec4(   pbr_out.color.rgb , alpha_mask_value.r)  ;
      

      // Implement alpha masking
    if (alpha_mask_value.r < 0.1) { // Use your threshold value here
        discard;
    }
    
    return final_color;
    
}
 