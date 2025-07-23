const GAMMA_CORRECTION: f32 = 1.2;
const WATER_THRESHOLD: f32 = 0.3;
const SAND_THRESHOLD: f32 = 0.4;
const GRASS_THRESHOLD: f32 = 0.6;
const ROCK_THRESHOLD: f32 = 0.8;

// 2D height map: row‐major Vec<Vec<f32>> of size N×N
// access as `map[y][x]`.
pub type HeightMap2D = Vec<Vec<f32>>;

// flatten a 2D height map (row‐major) into a single Vec<f32>
// For storing into MongoDB as a flat array
// For converting to an image buffer (e.g. grayscale u8) in the UI
pub fn flatten2(map: &HeightMap2D) -> Vec<f32> {
    map.iter().flat_map(|row| row.iter().cloned()).collect()
}

// Linearly interpolate between two RGB triples
fn lerp_color(a: [u8; 3], b: [u8; 3], t: f32) -> [u8; 3] {
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * t) as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * t) as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * t) as u8,
    ]
}

// Map a height in [0.0,1.0] to a realistic terrain color
fn height_to_rgb(h: f32) -> [u8; 3] {
    match h {
        x if x < WATER_THRESHOLD => {
            let t = x / WATER_THRESHOLD;
            lerp_color([0, 0, 128], [0, 128, 255], t) // deep to shallow water
        }
        x if x < SAND_THRESHOLD => {
            let t = (x - WATER_THRESHOLD) / (SAND_THRESHOLD - WATER_THRESHOLD);
            lerp_color([194, 178, 128], [220, 200, 160], t) // sand
        }
        x if x < GRASS_THRESHOLD => {
            let t = (x - SAND_THRESHOLD) / (GRASS_THRESHOLD - SAND_THRESHOLD);
            lerp_color([34, 139, 34], [50, 205, 50], t) // grass
        }
        x if x < ROCK_THRESHOLD => {
            let t = (x - SAND_THRESHOLD) / (ROCK_THRESHOLD - GRASS_THRESHOLD);
            lerp_color([128, 128, 128], [192, 192, 192], t) // rock
        }
        x => {
            let t = (x - ROCK_THRESHOLD) / (1.0 - ROCK_THRESHOLD);
            lerp_color([220, 220, 220], [255, 255, 255], t) // snow
        }
    }
}

// Convert a flat &[f32] into an RGB byte buffer
pub fn to_terrain_image(flat: &[f32], _size: usize) -> Vec<u8> {
    let mut buf = Vec::with_capacity(flat.len() * 3);
    for &h in flat {
        let [r, g, b] = height_to_rgb(h);
        buf.extend_from_slice(&[r, g, b]);
    }
    buf
}

// Normalize the final warped terrain
pub fn normalize2(map: &mut HeightMap2D) {
    let mut min = f32::MAX;
    let mut max = f32::MIN;

    for row in map.iter() {
        for &val in row.iter() {
            min = min.min(val);
            max = max.max(val);
        }
    }

    let range = (max - min).max(0.001); // prevent zero-division
    for row in map.iter_mut() {
        for val in row.iter_mut() {
            // normalize
            *val = (*val - min) / range;

            // Gamma curve for contrast boost
            *val = val.powf(GAMMA_CORRECTION);
        }
    }
}
