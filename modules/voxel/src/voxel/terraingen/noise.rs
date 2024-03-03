use bevy::math::{IVec3, Vec2, Vec2Swizzles, Vec3, Vec3Swizzles};
use noise::{utils::NoiseMapBuilder, MultiFractal};

pub fn rand2to1(p: Vec2, dot: Vec2) -> f32 {
    let sp: Vec2 = p.to_array().map(|x| x.sin()).into();
    let random = sp.dot(dot);
    (random.sin() * 143_758.55).fract()
}

#[inline(always)]
pub fn rand2to2(p: Vec2) -> Vec2 {
    Vec2::new(
        rand2to1(p, Vec2::new(12.989, 78.233)),
        rand2to1(p, Vec2::new(39.346, 11.135)),
    )
}

pub fn rand2to1i(vec: Vec2) -> f32 {
    let mut p3 = (vec.xyx() * 0.39).fract();
    p3 += p3.dot(p3.yzx());
    (p3.x + p3.y) * p3.z.fract()
}

#[allow(dead_code)]
#[inline(always)]
pub fn rand2to3(p: Vec2) -> Vec3 {
    Vec3::new(
        rand2to1(p, Vec2::new(12.989, 78.233)),
        rand2to1(p, Vec2::new(39.346, 11.135)),
        rand2to1(p, Vec2::new(73.156, 52.235)),
    )
}

#[allow(dead_code)]
#[inline(always)]
pub fn rand1dto1d(p: f32, mutator: f32) -> f32 {
    let random = (p + mutator).sin();
    (random * 143_758.55).fract()
}

#[allow(dead_code)]
#[inline(always)]
pub fn rand1to3(p: f32) -> Vec3 {
    Vec3::new(
        rand1dto1d(p, 3.9812),
        rand1dto1d(p, 1.2345),
        rand1dto1d(p, 5.4321),
    )
}

// This was ported from the code at https://www.ronja-tutorials.com/post/028-voronoi-noise/
pub fn voronoi(p: Vec2) -> Vec2 {
    const NEIGHBOUR_RANGE: i32 = 2; // A neighbour range of 1 will generate some weird artifacts lets use 2.

    let base_cell = p.floor();
    let mut closest_point = base_cell;
    let mut min_distance = 1f32;

    for x in -NEIGHBOUR_RANGE..=NEIGHBOUR_RANGE {
        for y in -NEIGHBOUR_RANGE..=NEIGHBOUR_RANGE {
            let cell = base_cell + Vec2::new(x as f32, y as f32);
            let cell_pos = cell + rand2to2(cell);
            let distance = (cell_pos - p).length_squared(); // using non squarred length to increase the throughput (a bit)

            if distance < min_distance {
                min_distance = distance;
                closest_point = cell;
            }
        }
    }

    closest_point
}

pub fn generate_heightmap_data(key: IVec3, chunk_len: usize) -> Vec<f32> {
    let noise = noise::Fbm::<noise::SuperSimplex>::new(0)
        .set_octaves(4)
        .set_frequency(0.005)
        .set_persistence(0.5)
        .set_lacunarity(2.0);

    noise::utils::PlaneMapBuilder::<_, 2>::new(noise)
        .set_size(chunk_len, chunk_len)
        .set_x_bounds(key.x as f64, (key.x + chunk_len as i32) as f64)
        .set_y_bounds(key.z as f64, (key.z + chunk_len as i32) as f64)
        .build()
        .into_iter()
        .map(|x| x.mul_add(20f64, 132f64) as f32)
        .collect()
}

/// A view into a slice of noise values with W x H dimensions.
/// Provides methods for fetching a value at specified coordinates and to map values to a range.
#[derive(Clone, Copy)]
pub struct Heightmap<'a, const W: usize, const H: usize> {
    slice: &'a [f32],
}

impl<'a, const W: usize, const H: usize> Heightmap<'a, W, H> {
    /// Gets the value at the specified coordinates.
    #[inline]
    pub fn get(&self, pos: [u32; 2]) -> u32 {
        self.slice[pos[1] as usize * W + pos[0] as usize].round() as u32
    }

    /// Constructs a view into a slice of noise values with W x H dimensions.
    #[inline]
    pub const fn from_slice(slice: &'a [f32]) -> Self {
        Self { slice }
    }
}
