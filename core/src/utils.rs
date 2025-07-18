// 2D height map: row‐major Vec<Vec<f32>> of size N×N
// access as `map[y][x]`.
pub type HeightMap2D = Vec<Vec<f32>>;

// flatten a 2D height map (row‐major) into a single Vec<f32>
// For storing into MongoDB as a flat array
// For converting to an image buffer (e.g. grayscale u8) in the UI
pub fn flatten2(map: &HeightMap2D) -> Vec<f32> {
    map.iter().flat_map(|row| row.iter().cloned()).collect()
}

// placeholder for flattening a 3D volume
pub fn flatten3(_vol: &Vec<Vec<Vec<f32>>>) -> Vec<f32> {
    unimplemented!("flatten3 will be implemented when 3D is fully supported");
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
        x if x < 0.3 => {
            let t = x / 0.3;
            lerp_color([0, 0, 128], [0, 128, 255], t) // deep to shallow water
        }
        x if x < 0.4 => {
            let t = (x - 0.3) / 0.1;
            lerp_color([194, 178, 128], [220, 200, 160], t) // sand
        }
        x if x < 0.6 => {
            let t = (x - 0.4) / 0.2;
            lerp_color([34, 139, 34], [50, 205, 50], t) // grass
        }
        x if x < 0.8 => {
            let t = (x - 0.6) / 0.2;
            lerp_color([128, 128, 128], [192, 192, 192], t) // rock
        }
        x => {
            let t = (x - 0.8) / 0.2;
            lerp_color([220, 220, 220], [255, 255, 255], t) // snow
        }
    }
}

// Convert a flat &[f32] into an RGB byte buffer
pub fn to_terrain_image(flat: &[f32], size: usize) -> Vec<u8> {
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
            if val < min {
                min = val;
            }
            if val > max {
                max = val;
            }
        }
    }

    let range = max - min;
    if range > 0.0 {
        for row in map.iter_mut() {
            for val in row.iter_mut() {
                *val = (*val - min) / range;
            }
        }
    }
}
