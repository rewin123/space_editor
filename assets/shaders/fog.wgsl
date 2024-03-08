
const FOG_MIN_DISTANCE: f32 = 384.0;
const FOG_COLOR: vec4<f32> = vec4<f32>(0.4, 0.4, 0.4, 1.0);

// 
fn ffog_calc_factor(clamped_d: f32, fog_distance: f32, chunk_size: f32) -> f32 {
    let fog_max: f32 = fog_distance;
    let fog_min: f32 = fog_max - 4.5 * chunk_size;
    let new_clamped_d = clamp(fog_min, clamped_d, fog_max);
    return 1.0 - (fog_max - new_clamped_d) / (fog_max - fog_min);
}

fn ffog_apply_fog(d: f32, fog_min: f32, chunk_size: f32, color: vec4<f32>) -> vec4<f32> {
    return mix(color, FOG_COLOR, ffog_calc_factor(d, fog_min, 32.0));
}