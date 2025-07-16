use crate::NoiseGenerator;

// 2D Simplex noise generator with multiple octaves
// Based on Ken Perlin's Simplex algorithm
pub struct Simplex2D {
    seed: u64,
    frequency: f64,
    persistence: f64,
    octaves: usize,
    perm: [u8; 512],
    // Simplex divides space into triangles, rather than squares
    // This results in better isotropy (uniformity in all directions)
    grad3: [(i8, i8); 12],
}

impl Simplex2D {
    pub fn new(seed: u64, frequency: f64, persistence: f64, octaves: usize) -> Self {
        // Same permutation‐table construction as Perlin2D:
        let mut p: Vec<u8> = (0..256).map(|i| i as u8).collect();
        let mut x = seed ^ 0x1234_5678_9ABC_DEF0_u64;
        let mut rng = || {
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            (x & 0xFF) as u8
        };
        for i in (1..256).rev() {
            let j = (rng() as usize) % (i + 1);
            p.swap(i, j);
        }
        let mut perm = [0u8; 512];
        for i in 0..512 {
            perm[i] = p[i & 255];
        }

        // Predefined 2D gradient directions (normalized to length ≈1.0):
        let grad3 = [
            (1, 1),
            (-1, 1),
            (1, -1),
            (-1, -1),
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 2),
            (-1, 2),
            (1, -2),
            (-1, -2),
        ];

        Self {
            seed,
            frequency,
            persistence,
            octaves,
            perm,
            grad3,
        }
    }

    // Dot product helper chooses a gradient from grad3[hash % 12]
    #[inline]
    fn dot(g: (i8, i8), x: f64, y: f64) -> f64 {
        (g.0 as f64) * x + (g.1 as f64) * y
    }

    // Raw 2D Simplex noise at (xin, yin)
    // Returns in range [−1.0, +1.0], roughly
    fn raw_noise(&self, xin: f64, yin: f64) -> f64 {
        // Approximate value of sqrt(3)
        const SQRT_3: f64 = 1.732_050_807_568_877_293_5;
        // Skewing/Unskewing factors for 2D simplex
        const F2: f64 = 0.5 * (SQRT_3 - 1.0); // comresses the square into a rhombus made of equilateral triangles
        const G2: f64 = (3.0 - SQRT_3) / 6.0; // reverses the skewing

        // Skew input space to determine simplex cell
        let s = (xin + yin) * F2; // Skew factor
        // Coordinates of the triangle we are in
        let i = (xin + s).floor() as i32;
        let j = (yin + s).floor() as i32;

        // Unskew back to get the relative position to the origin corner
        let t = (i + j) as f64 * G2; // Unskew factor
        // Local coordinates within the simplex triangle
        let x0 = xin - (i as f64 - t);
        let y0 = yin - (j as f64 - t);

        // Determine which simplex triangle we are in (Lower or Upper)
        let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };

        // Offsets for remaining corners
        let x1 = x0 - i1 as f64 + G2;
        let y1 = y0 - j1 as f64 + G2;
        let x2 = x0 - 1.0 + 2.0 * G2;
        let y2 = y0 - 1.0 + 2.0 * G2;

        // Hash the three simplex corners
        let ii = (i & 255) as usize;
        let jj = (j & 255) as usize;
        // Double lookup ensures hashing produces indexes based on both i and j
        let gi0 = (self.perm[ii + self.perm[jj] as usize] as usize) % 12;
        let gi1 = (self.perm[ii + i1 + self.perm[jj + j1] as usize] as usize) % 12;
        let gi2 = (self.perm[ii + 1 + self.perm[jj + 1] as usize] as usize) % 12;

        // Contribution from corner 0
        let mut n0 = 0.0;
        let t0 = 0.5 - x0 * x0 - y0 * y0; // Circular distance of infuence
        if t0 > 0.0 {
            let t0_sq = t0 * t0;
            n0 = t0_sq * t0_sq * Self::dot(self.grad3[gi0], x0, y0);
        }
        // Corner 1
        let mut n1 = 0.0;
        let t1 = 0.5 - x1 * x1 - y1 * y1;
        if t1 > 0.0 {
            let t1_sq = t1 * t1;
            n1 = t1_sq * t1_sq * Self::dot(self.grad3[gi1], x1, y1);
        }
        // Corner 2
        let mut n2 = 0.0;
        let t2 = 0.5 - x2 * x2 - y2 * y2;
        if t2 > 0.0 {
            let t2_sq = t2 * t2;
            n2 = t2_sq * t2_sq * Self::dot(self.grad3[gi2], x2, y2);
        }

        // The result is scaled to return roughly [-1,1] to make it consistent
        70.0 * (n0 + n1 + n2)
    }

    pub fn generate(&self, size: usize) -> Vec<Vec<f32>> {
        let mut data = vec![vec![0.0; size]; size];
        for y in 0..size {
            for x in 0..size {
                let nx = x as f64 / size as f64;
                let ny = y as f64 / size as f64;
                data[y][x] = self.get2(nx, ny) as f32;
            }
        }
        data
    }
}

impl NoiseGenerator for Simplex2D {
    fn get2(&self, x: f64, y: f64) -> f64 {
        let mut amplitude = 1.0;
        let mut freq = self.frequency;
        let mut total = 0.0;
        let mut max_amp = 0.0;

        for _ in 0..self.octaves {
            total += self.raw_noise(x * freq, y * freq) * amplitude;
            max_amp += amplitude;
            amplitude *= self.persistence;
            freq *= 2.0;
        }

        // Normalize to [-1, 1]
        total / max_amp
    }
}

#[cfg(test)]
mod tests {
    use crate::NoiseGenerator;

    use super::Simplex2D;

    #[test]
    fn simplex2_determinism() {
        let s1 = Simplex2D::new(9999, 0.05, 0.5, 4);
        let s2 = Simplex2D::new(9999, 0.05, 0.5, 4);
        let a = s1.get2(1.23, 4.56);
        let b = s2.get2(1.23, 4.56);
        assert!((a - b).abs() < 1e-12);
    }

    #[test]
    fn simplex2_range() {
        let s = Simplex2D::new(0, 0.1, 0.5, 6);
        for &(x, y) in &[(0.0, 0.0), (5.5, -5.5), (100.1, 100.1)] {
            let v = s.get2(x, y);
            assert!(v >= -1.0 - 1e-6 && v <= 1.0 + 1e-6);
        }
    }

    #[test]
    #[should_panic]
    fn simplex2_get3_panic() {
        let s = Simplex2D::new(0, 0.1, 0.5, 4);
        let _ = s.get3(1.0, 2.0, 3.0);
    }
}
