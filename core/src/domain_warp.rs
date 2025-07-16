use crate::{NoiseGenerator, utils::HeightMap2D};

pub struct DomainWarp2D<'a> {
    pub base: &'a dyn NoiseGenerator,
    pub warp: &'a dyn NoiseGenerator,
    pub size: usize,
    pub warp_strength: f64,
}

impl<'a> DomainWarp2D<'a> {
    pub fn generate(&self) -> HeightMap2D {
        let mut map = vec![vec![0.0; self.size]; self.size];
        for y in 0..self.size {
            for x in 0..self.size {
                let fx = x as f64 / self.size as f64;
                let fy = y as f64 / self.size as f64;

                let dx = self.warp.get2(fx * 3.0, fy * 3.0);
                let dy = self.warp.get2((fx + 5.2) * 3.0, (fy + 5.2) * 3.0);

                let warped_x = (fx + dx * self.warp_strength).clamp(0.0, 1.0);
                let warped_y = (fy + dy * self.warp_strength).clamp(0.0, 1.0);

                map[y][x] = self.base.get2(warped_x, warped_y) as f32;
            }
        }
        map
    }
}
