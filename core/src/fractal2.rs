use crate::NoiseGenerator;
use crate::utils::HeightMap2D;

// 2D fractal terrain generator using the Diamond–Square algorithm
pub struct Fractal2D {
    size: usize, // must be 2^n + 1, e.g. 129, 257
    seed: u64,
    roughness: f64, // controls how much random offset decreases each step
    map: HeightMap2D,
}

impl Fractal2D {
    pub fn new(size: usize, seed: u64, roughness: f64) -> Self {
        assert!(
            size >= 3 && (size - 1).is_power_of_two(),
            "size must be 2^n+1"
        );

        Self {
            size,
            seed,
            roughness,
            map: vec![vec![0.0f32; size]; size],
        }
    }

    // Generate and return a size×size height‐map with values in roughly [−1, +1]
    pub fn generate(&mut self) -> HeightMap2D {
        let mut map = vec![vec![0.0f32; self.size]; self.size];
        // Simple xorshift RNG for reproducible randomness
        let mut x = self.seed ^ 0xCAFEBABE12345678;
        let mut rng = || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            // map to [−1, +1]
            ((x as f64 / u64::MAX as f64) * 2.0 - 1.0) as f32
        };

        // Initialize corners
        map[0][0] = rng();
        map[0][self.size - 1] = rng();
        map[self.size - 1][0] = rng();
        map[self.size - 1][self.size - 1] = rng();

        // Step is the current distance between 2 know points
        let mut step = self.size - 1;
        // Offset is the amplitude of the random noise
        // It decreases with each step to create a fractal pattern
        let mut offset = 1.0;

        while step > 1 {
            // Distance to the center from a corner
            let half = step / 2;

            // Diamond step
            for y in (0..self.size - 1).step_by(step) {
                for x in (0..self.size - 1).step_by(step) {
                    // Calculate the average height of the four corners
                    let avg =
                        (map[y][x] + map[y][x + step] + map[y + step][x] + map[y + step][x + step])
                            * 0.25;
                    // Set the center point to the average plus some random offset
                    map[y + half][x + half] = avg + rng() * offset;
                }
            }

            // Square step
            for y in (0..self.size).step_by(half) {
                for x in ((y + half)..self.size).step_by(step) {
                    let mut sum = 0.0;
                    let mut cnt = 0;
                    if x >= half {
                        sum += map[y][x - half];
                        cnt += 1;
                    }
                    if x + half < self.size {
                        sum += map[y][x + half];
                        cnt += 1;
                    }
                    if y >= half {
                        sum += map[y - half][x];
                        cnt += 1;
                    }
                    if y + half < self.size {
                        sum += map[y + half][x];
                        cnt += 1;
                    }
                    let avg = sum / cnt as f32;
                    map[y][x] = avg + rng() * offset;
                }
            }

            step = half;
            offset *= self.roughness as f32;
        }

        // Store it for get2()
        self.map = map.clone();
        map
    }
}

impl NoiseGenerator for Fractal2D {
    fn get2(&self, x: f64, y: f64) -> f64 {
        // Use bilinear sampling from a pre-generated map:
        let fx = x * self.size as f64;
        let fy = y * self.size as f64;
        let xi = fx.floor() as usize;
        let yi = fy.floor() as usize;

        if xi + 1 >= self.size || yi + 1 >= self.size {
            return 0.0;
        }

        let tx = (fx - xi as f64) as f32;
        let ty = (fy - yi as f64) as f32;

        let a = self.map[yi][xi];
        let b = self.map[yi][xi + 1];
        let c = self.map[yi + 1][xi];
        let d = self.map[yi + 1][xi + 1];

        let ab = a * (1.0 - tx) + b * tx;
        let cd = c * (1.0 - tx) + d * tx;
        let val = ab * (1.0 - ty) + cd * ty;

        val as f64
    }
}

#[cfg(test)]
mod tests {
    use super::Fractal2D;

    #[test]
    fn fractal2_dimensions() {
        let mut f = Fractal2D::new(129, 0, 1.0);
        let m = f.generate();
        assert_eq!(m.len(), 129);
        assert_eq!(m[0].len(), 129);
    }

    #[test]
    fn fractal2_determinism() {
        let mut f1 = Fractal2D::new(65, 42, 0.8);
        let mut f2 = Fractal2D::new(65, 42, 0.8);
        assert_eq!(f1.generate(), f2.generate());
    }

    #[test]
    fn fractal2_value_range() {
        let mut f = Fractal2D::new(33, 7, 0.5);
        let m = f.generate();
        for row in &m {
            for &v in row {
                assert!(v >= -2.0 && v <= 2.0, "value {} out of expected range", v);
            }
        }
    }
}
