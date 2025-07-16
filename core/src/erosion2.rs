use crate::utils::HeightMap2D;

pub struct ThermalErosion2D {
    iterations: usize,
    talus_angle: f32, // maximum stable slope before material moves
}

impl ThermalErosion2D {
    // iterations - how many passes to run
    // More iterations = smoother terrain.
    // talus_angle - slope threshold (e.g. 1.0)
    // if the slope between a cell and its neighbor exceeds this angle,
    // material will errode downhill.
    pub fn new(iterations: usize, talus_angle: f32) -> Self {
        Self {
            iterations,
            talus_angle,
        }
    }

    // In‐place apply erosion to the height‐map
    pub fn apply(&self, map: &mut HeightMap2D) {
        let h = map.len();
        let w = map[0].len();

        for _ in 0..self.iterations {
            // Accumulate deltas here to avoid order bias
            let mut delta = vec![vec![0.0f32; w]; h];

            for y in 0..h {
                for x in 0..w {
                    let curr = map[y][x];
                    // Check 4‐neighbors
                    let mut max_diff = 0.0; // Largest downhill slope
                    let mut max_n = (0, 0); // Neighbor with the largest downhill slope
                    // Use & for borrowing to avoid copying
                    for &(dy, dx) in &[(0, 1), (1, 0), (0, -1), (-1, 0)] {
                        let ny = y as isize + dy;
                        let nx = x as isize + dx;
                        if ny >= 0 && ny < h as isize && nx >= 0 && nx < w as isize {
                            let v = map[ny as usize][nx as usize];
                            let diff = curr - v; // Elevation difference
                            if diff > max_diff {
                                max_diff = diff;
                                max_n = (ny as usize, nx as usize);
                            }
                        }
                    }
                    // If slope exceeds talus errode
                    if max_diff > self.talus_angle {
                        let amount = (max_diff - self.talus_angle) * 0.5;
                        delta[y][x] -= amount; // Current cell loses height
                        delta[max_n.0][max_n.1] += amount; // The steepest downhill gain height
                    }
                }
            }

            // Apply all deltas:
            for y in 0..h {
                for x in 0..w {
                    map[y][x] += delta[y][x];
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ThermalErosion2D;

    #[test]
    fn erosion2_simple_peak() {
        // 3×3 map: peak at center
        let mut map = vec![
            vec![0.0, 0.0, 0.0],
            vec![0.0, 2.0, 0.0],
            vec![0.0, 0.0, 0.0],
        ];
        let er = ThermalErosion2D::new(1, 1.0);
        er.apply(&mut map);
        // Center should decrease, at least one neighbor should increase
        assert!(map[1][1] < 2.0);
        assert!(map[1][0] > 0.0 || map[1][2] > 0.0 || map[0][1] > 0.0 || map[2][1] > 0.0);
    }

    #[test]
    fn erosion2_determinism() {
        let mut m1: Vec<Vec<f32>> = (0..5).map(|i| vec![i as f32; 5]).collect();
        let mut m2 = m1.clone();
        let er = ThermalErosion2D::new(3, 0.5);
        er.apply(&mut m1);
        let er2 = ThermalErosion2D::new(3, 0.5);
        er2.apply(&mut m2);
        assert_eq!(m1, m2);
    }
}
