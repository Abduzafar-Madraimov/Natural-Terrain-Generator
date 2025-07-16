use crate::NoiseGenerator;

// This is similar to Perlin2D but extended to 3D
pub struct Perlin3D {
    seed: u64,
    frequency: f64,
    persistence: f64,
    octaves: usize,
    perm: [u8; 512],
}

impl Perlin3D {
    pub fn new(seed: u64, frequency: f64, persistence: f64, octaves: usize) -> Self {
        // Build permutation table exactly as in Perlin2D
        let mut p: Vec<u8> = (0..256).map(|i| i as u8).collect();
        let mut x = seed ^ 0xAABBCCDDEEFF1122_u64;
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

        Self {
            seed,
            frequency,
            persistence,
            octaves,
            perm,
        }
    }

    #[inline]
    fn fade(t: f64) -> f64 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    #[inline]
    fn lerp(a: f64, b: f64, t: f64) -> f64 {
        a + t * (b - a)
    }

    // Gradient function for 3D: based on hashed value hi, choose from 12 directions
    #[inline]
    fn grad(hash: u8, x: f64, y: f64, z: f64) -> f64 {
        // Convert lower 4 bits of hash into 12 gradient directions
        let h = (hash & 0xF) as usize;
        let u = if h < 8 { x } else { y };
        let v = if h < 4 {
            y
        } else if h == 12 || h == 14 {
            x
        } else {
            z
        };
        let sign_u = if (h & 1) == 0 { u } else { -u };
        let sign_v = if (h & 2) == 0 { v } else { -v };
        sign_u + sign_v
    }

    // Raw single‐octave Perlin noise at (x, y, z).
    fn noise(&self, x: f64, y: f64, z: f64) -> f64 {
        // Find unit cube that contains point
        let xi = x.floor() as i32 & 255;
        let yi = y.floor() as i32 & 255;
        let zi = z.floor() as i32 & 255;
        // Relative coordinates within cube
        let xf = x - x.floor();
        let yf = y - y.floor();
        let zf = z - z.floor();
        // Fade curves for each
        let u = Self::fade(xf);
        let v = Self::fade(yf);
        let w = Self::fade(zf);

        // Hash corners of the cube
        let aaa = self.perm[(self.perm[(self.perm[xi as usize] as usize + yi as usize) & 255]
            as usize
            + zi as usize)
            & 255];
        let aba = self.perm[(self.perm
            [(self.perm[xi as usize] as usize + ((yi + 1) & 255) as usize) & 255]
            as usize
            + zi as usize)
            & 255];
        let aab = self.perm[(self.perm[(self.perm[xi as usize] as usize + yi as usize) & 255]
            as usize
            + ((zi + 1) & 255) as usize)
            & 255];
        let abb = self.perm[(self.perm
            [(self.perm[xi as usize] as usize + ((yi + 1) & 255) as usize) & 255]
            as usize
            + ((zi + 1) & 255) as usize)
            & 255];
        let baa = self.perm[(self.perm
            [(self.perm[((xi + 1) & 255) as usize] as usize + yi as usize) & 255]
            as usize
            + zi as usize)
            & 255];
        let bba = self.perm[(self.perm
            [(self.perm[((xi + 1) & 255) as usize] as usize + ((yi + 1) & 255) as usize) & 255]
            as usize
            + zi as usize)
            & 255];
        let bab = self.perm[(self.perm
            [(self.perm[((xi + 1) & 255) as usize] as usize + yi as usize) & 255]
            as usize
            + ((zi + 1) & 255) as usize)
            & 255];
        let bbb = self.perm[(self.perm
            [(self.perm[((xi + 1) & 255) as usize] as usize + ((yi + 1) & 255) as usize) & 255]
            as usize
            + ((zi + 1) & 255) as usize)
            & 255];

        // Compute gradient contributions
        let x1 = Self::lerp(
            Self::grad(aaa, xf, yf, zf),
            Self::grad(baa, xf - 1.0, yf, zf),
            u,
        );
        let x2 = Self::lerp(
            Self::grad(aba, xf, yf - 1.0, zf),
            Self::grad(bba, xf - 1.0, yf - 1.0, zf),
            u,
        );
        let y1 = Self::lerp(x1, x2, v);

        let x3 = Self::lerp(
            Self::grad(aab, xf, yf, zf - 1.0),
            Self::grad(bab, xf - 1.0, yf, zf - 1.0),
            u,
        );
        let x4 = Self::lerp(
            Self::grad(abb, xf, yf - 1.0, zf - 1.0),
            Self::grad(bbb, xf - 1.0, yf - 1.0, zf - 1.0),
            u,
        );
        let y2 = Self::lerp(x3, x4, v);

        // Final interpolation along z:
        Self::lerp(y1, y2, w)
    }
}

impl NoiseGenerator for Perlin3D {
    fn get3(&self, x: f64, y: f64, z: f64) -> f64 {
        let mut amplitude = 1.0;
        let mut freq = self.frequency;
        let mut total = 0.0;
        let mut max_amp = 0.0;

        for _ in 0..self.octaves {
            total += self.noise(x * freq, y * freq, z * freq) * amplitude;
            max_amp += amplitude;
            amplitude *= self.persistence;
            freq *= 2.0;
        }

        total / max_amp
    }
}

#[cfg(test)]
mod tests {
    use crate::NoiseGenerator;

    use super::Perlin3D;

    #[test]
    fn perlin3_determinism() {
        let p1 = Perlin3D::new(2025, 0.02, 0.5, 3);
        let p2 = Perlin3D::new(2025, 0.02, 0.5, 3);
        let a = p1.get3(1.23, 4.56, 7.89);
        let b = p2.get3(1.23, 4.56, 7.89);
        assert!((a - b).abs() < 1e-12);
    }

    #[test]
    fn perlin3_range() {
        let p = Perlin3D::new(0, 0.1, 0.5, 5);
        for &(x, y, z) in &[(0.0, 0.0, 0.0), (1.5, -2.5, 3.5), (100.1, 200.2, -50.3)] {
            let v = p.get3(x, y, z);
            assert!(v >= -1.0 - 1e-6 && v <= 1.0 + 1e-6);
        }
    }

    #[test]
    #[should_panic]
    fn perlin3_get2_panic() {
        let p = Perlin3D::new(0, 0.1, 0.5, 3);
        let _ = p.get2(1.0, 2.0);
    }
}
