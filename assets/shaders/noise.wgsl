
// Shader code underneath this is extracted from the veloren project shader assets and as such is licensed under the project GPLv3 license
// See https://github.com/veloren/veloren

fn hash(p: vec4<f32>) -> f32 {
    var n: vec4<f32> = fract(p * 0.3183099 + 0.1) - fract(p + 23.22121);
    n = n * 17.0;
    return (fract(n.x * n.y * (1.0 - n.z) * n.w * (n.x + n.y + n.z + n.w)) - 0.5) * 2.0;
}