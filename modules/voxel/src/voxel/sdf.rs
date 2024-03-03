//
// This is a rust port of SDF shape functions from Inigo Quilez's web site available at the following URL :
//   https://iquilezles.org/articles/distfunctions/
//

use bevy::math::{vec2, Vec2, Vec3, Vec3Swizzles};

#[allow(dead_code)]
pub fn sdf_sphere(p: Vec3, radius: f32) -> f32 {
    p.length() - radius
}

#[allow(dead_code)]
pub fn sdf_torus(p: Vec3, t: Vec2) -> f32 {
    let q = vec2(p.xz().length() - t.x, p.y);
    q.length() - t.y
}

#[allow(dead_code)]
#[allow(clippy::manual_clamp)]
pub fn sdf_capped_cylinder(p: Vec3, h: f32, r: f32) -> f32 {
    let d = vec2(p.xz().length(), p.y).abs() - vec2(h, r);
    d.x.max(d.y).min(0.) + d.max(Vec2::ZERO).length()
}

#[allow(dead_code)]
#[allow(clippy::manual_clamp)]
pub fn sdf_box(p: Vec3, b: Vec3) -> f32 {
    let q = p.abs() - b;
    q.x.max(q.y).max(q.z).min(0.) + q.max(Vec3::ZERO).length()
}

#[allow(dead_code)]
pub fn sdf_v_capsule(p: Vec3, h: f32, r: f32) -> f32 {
    let mut pp = p;
    pp.y -= p.y.clamp(0.0, h);
    pp.length() - r
}

#[allow(clippy::suboptimal_flops)]
pub fn sdf_vcone(p: Vec3, r: f32, h: f32) -> f32 {
    let slope = -h / r;
    let ratio = (1.0 / slope * slope + 1.0).sqrt();
    let t = vec2(p.xz().length(), p.y);
    let v = slope.mul_add(t.x, h) - t.y;
    let p = v * ratio;
    -p.min(t.y)
}
