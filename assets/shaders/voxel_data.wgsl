

//
// Layout of voxel information encoded into a single u32
//
//  00000000    00000000    00000000    00000000    
//  XXXXXYYY    YYZZZZZ          NNN    MATERIAL
//
// X: X position
// Y: Y position
// Z: Z position
// N: normal index in the VOXEL_NORMALS array
// MATERIAL: material index in the palette
// 
// The remaining 5 free bits could be used to store UV data or additional info or even extend voxel material id size.

// An array of voxel face normals 
var<private> VOXEL_NORMALS: array<vec3<f32>, 6> = array<vec3<f32>, 6>(
    vec3<f32>(-1., 0., 0.),
    vec3<f32>(0., -1., 0.),
    vec3<f32>(0., 0., -1.), 
    vec3<f32>(1., 0., 0.), 
    vec3<f32>(0., 1., 0.), 
    vec3<f32>(0., 0., 1.), 
);

// Extracts the normal face index from the encoded voxel data
fn voxel_data_extract_normal(voxel_data: u32) -> vec3<f32> {
    return VOXEL_NORMALS[voxel_data >> 8u & 7u];
}

// fn voxel_data_extract_position(voxel_data: u32) -> vec3<f32> {
//     return vec3<f32>(
//         f32(voxel_data >> 27u),
//         f32(voxel_data >> 22u & 31u),
//         f32(voxel_data >> 17u & 31u)
//     );
// }

// Extracts the material index from the encoded voxel data
fn voxel_data_extract_material_index(voxel_data: u32) -> u32 {
    return voxel_data & 255u;
}
