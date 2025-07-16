// core holds all the noise, fractal, erosion algorithms
pub mod domain_warp;
pub mod erosion2;
pub mod fractal2;
pub mod perlin2;
pub mod perlin3;
pub mod simplex2;
pub mod utils;

pub use erosion2::ThermalErosion2D;
pub use fractal2::Fractal2D;
pub use perlin2::Perlin2D;
pub use perlin3::Perlin3D;
pub use simplex2::Simplex2D;
pub use utils::flatten2;

// noise generator that can sample 2D or 3D points
// 2D‐only implementations override `get2(...)`.
// 3D‐only implementations override `get3(...)`.
pub trait NoiseGenerator {
    // Sample 2D noise at (x, y).
    fn get2(&self, x: f64, y: f64) -> f64 {
        panic!("get2 not implemented for this generator");
    }

    // Sample 3D noise at (x, y, z).
    fn get3(&self, x: f64, y: f64, z: f64) -> f64 {
        panic!("get3 not implemented for this generator");
    }
}
