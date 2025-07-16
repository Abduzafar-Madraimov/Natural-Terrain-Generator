use crate::NoiseGenerator;

// 2D Perlin Noise generator with support for multiple octaves
pub struct Perlin2D {
    seed: u64,        // Arbitrary u64 seed
    frequency: f64,   // Controls the "zoom level" of the noise pattern
    persistence: f64, // Controls amplitude scaling per octave
    octaves: usize,   // number of octaves to sum
    perm: [u8; 512],  // permutation table (256 duplicated)
}

impl Perlin2D {
    pub fn new(seed: u64, frequency: f64, persistence: f64, octaves: usize) -> Self {
        // build a pseudorandom permutation table of size 256, duplicated into 512
        let mut p: Vec<u8> = (0..256).map(|i| i as u8).collect();
        // Simple xorshift-based (with a large constant) RNG for shuffling
        let mut x = seed ^ 0xDEADBEEFCAFEBABE_u64;
        let mut rng = || {
            // These values are known to work well for shuffling
            // Mulitple shift + XOR for better results
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            // Bitmasking the lowest 8 bits
            (x & 0xFF) as u8
        };
        // Fisher–Yates shuffle p[0..256]
        for i in (1..256).rev() {
            // mod (i + 1) to constrain it to [0..i]
            let j = (rng() as usize) % (i + 1);
            // to place each element in a random position
            p.swap(i, j);
        }
        // Duplicate into an array of length 512
        // Instead of perm[(x + 1) % 256]
        // To avoid costly modulo operations when doing lookups
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

    // Fade function as defined by Ken Perlin: 6t^5 − 15t^4 + 10t^3
    // It helps to avoid visual artifacts by smoothing the interpolation
    // As its first and second derivatives are zero at t=0 and t=1
    // First derivative - slope; Second derivative - curvature
    #[inline] // To skip function call overhead
    fn fade(t: f64) -> f64 {
        t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
    }

    // Linear inerpolation
    #[inline]
    fn lerp(a: f64, b: f64, t: f64) -> f64 {
        a + t * (b - a)
    }

    // Gradient function for 2D: based on the hashed value hi, choose a gradient direction
    #[inline]
    fn grad(hash: u8, x: f64, y: f64) -> f64 {
        // Convert low 4 bits of hash into 12 gradient directions
        let h = (hash & 0xF) as usize;
        // Predefined gradients (permute among (1,1), (1,-1), (-1,1), (-1,-1), (1,0), (-1,0), (0,1), (0,-1))
        let u = if h < 8 { x } else { y };
        let v = if h < 8 { y } else { x };
        let sign_u = if (h & 1) == 0 { u } else { -u };
        let sign_v = if (h & 2) == 0 { v } else { -v };
        // Equivalent ot a simplified dot product
        // Gives scalar influence of the gradient direction on the point (x, y)
        sign_u + sign_v
    }

    // Raw single‐octave Perlin noise at (x, y)
    // Returns in range ≈ [−√2, √2]
    fn noise(&self, x: f64, y: f64) -> f64 {
        // Find unit square that contains point (Which square to sample?)
        let xi = x.floor() as i32 & 255;
        let yi = y.floor() as i32 & 255;
        // Relative x/y within the square (Where within the square?)
        let xf = x - x.floor();
        let yf = y - y.floor();
        // Compute fade curves for x, y to get smooth interpolation
        let u = Self::fade(xf);
        let v = Self::fade(yf);

        // Hash coordinates of the four corners to get pseudo-random gradient indices
        let aa = self.perm[(self.perm[xi as usize] as usize + yi as usize) & 255];
        let ab = self.perm[(self.perm[xi as usize] as usize + ((yi + 1) & 255) as usize) & 255];
        let ba = self.perm[(self.perm[((xi + 1) & 255) as usize] as usize + yi as usize) & 255];
        let bb = self.perm
            [(self.perm[((xi + 1) & 255) as usize] as usize + ((yi + 1) & 255) as usize) & 255];

        // Compute gradients at each corner:
        let x1 = Self::lerp(Self::grad(aa, xf, yf), Self::grad(ba, xf - 1.0, yf), u);
        let x2 = Self::lerp(
            Self::grad(ab, xf, yf - 1.0),
            Self::grad(bb, xf - 1.0, yf - 1.0),
            u,
        );
        // Interpolate the two results along y
        Self::lerp(x1, x2, v)
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

impl NoiseGenerator for Perlin2D {
    // Return a multi-octave Perlin noise value at (x, y) (Fractal Brownian Motion)
    // The result is roughly in [−1.0, +1.0] after normalization
    fn get2(&self, x: f64, y: f64) -> f64 {
        let mut amplitude = 1.0; // Weight of the current octave
        let mut freq = self.frequency; // How zoomed in we are on the noise pattern
        let mut total = 0.0; // Accumulated noise value
        let mut max_amp = 0.0; // Maximum possible amplitude to normalize the result

        for _ in 0..self.octaves {
            total += self.noise(x * freq, y * freq) * amplitude;
            max_amp += amplitude;
            amplitude *= self.persistence;
            freq *= 2.0;
        }

        // Normalize to [−1, +1] to keep the output consistent
        total / max_amp
    }
}

#[cfg(test)]
mod tests {
    use crate::NoiseGenerator;

    use super::Perlin2D;

    #[test]
    fn perlin2_determinism() {
        let p1 = Perlin2D::new(1234, 0.01, 0.5, 4);
        let p2 = Perlin2D::new(1234, 0.01, 0.5, 4);
        // Same seed + params ⇒ same output
        let a = p1.get2(10.5, -3.7);
        let b = p2.get2(10.5, -3.7);
        assert!((a - b).abs() < 1e-12);
    }

    #[test]
    // Stays within [-1.0, 1.0] range
    fn perlin2_range() {
        let p = Perlin2D::new(0, 0.1, 0.5, 6);
        for &pt in &[(0.0, 0.0), (5.3, -1.2), (100.1, 200.2)] {
            let v = p.get2(pt.0, pt.1);
            assert!(v >= -1.0 - 1e-6 && v <= 1.0 + 1e-6);
        }
    }

    #[test]
    #[should_panic]
    fn perlin2_get3_panic() {
        let p = Perlin2D::new(0, 0.1, 0.5, 4);
        // Calling get3 on a 2D-only generator should panic
        let _ = p.get3(1.0, 2.0, 3.0);
    }
}
