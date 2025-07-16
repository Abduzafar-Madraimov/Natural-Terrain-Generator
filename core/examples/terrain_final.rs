use core::utils::HeightMap2D;
use core::{Fractal2D, ThermalErosion2D};
use image::{Rgb, RgbImage};
use palette::{Gradient, LinSrgb};
use std::path::Path;

// Compute simple hillshade for a height-map
// `z_scale` adjusts vertical exaggeration
fn hillshade(map: &HeightMap2D, z_scale: f32) -> Vec<Vec<f32>> {
    let h = map.len();
    let w = map[0].len();
    let mut shade = vec![vec![0.0; w]; h];
    let azimuth = std::f32::consts::PI / 4.0; // 45°
    let altitude = std::f32::consts::PI / 4.0; // 45°
    let (sin_alt, cos_alt) = altitude.sin_cos();

    for y in 1..h - 1 {
        for x in 1..w - 1 {
            // 3×3 neighborhood finite differences
            let dzdx = ((map[y][x + 1] - map[y][x - 1]) / 2.0) * z_scale;
            let dzdy = ((map[y + 1][x] - map[y - 1][x]) / 2.0) * z_scale;
            // Surface normal
            let nx = -dzdx;
            let ny = -dzdy;
            let nz = 1.0;
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            let (nx, ny, nz) = (nx / len, ny / len, nz / len);
            // Light vector from azimuth/altitude
            let lx = azimuth.cos() * cos_alt;
            let ly = azimuth.sin() * cos_alt;
            let lz = sin_alt;
            // Lambertian dot
            let val = (nx * lx + ny * ly + nz * lz).max(0.0);
            shade[y][x] = val;
        }
    }
    shade
}

fn main() {
    // Generate a large height-map
    let size = 513; // 2^9 + 1
    let mut terrain = Fractal2D::new(size, 2025, 1.0).generate();
    ThermalErosion2D::new(20, 1.0).apply(&mut terrain);

    // Compute hillshade
    let shade = hillshade(&terrain, 1.0);

    // Create a color gradient - deep water to beach to grass to rock to snow
    let gradient = Gradient::with_domain(vec![
        (0.00, LinSrgb::new(0.0, 0.0, 0.5)), // deep blue
        (0.30, LinSrgb::new(0.8, 0.8, 0.5)), // sand
        (0.50, LinSrgb::new(0.1, 0.6, 0.2)), // green
        (0.75, LinSrgb::new(0.5, 0.4, 0.3)), // rock
        (1.00, LinSrgb::new(1.0, 1.0, 1.0)), // snow
    ]);

    // Normalize terrain heights to 0.0..1.0 for coloring
    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;
    for row in &terrain {
        for &v in row {
            min = min.min(v);
            max = max.max(v);
        }
    }

    // Build final image
    let mut img = RgbImage::new(size as u32, size as u32);
    for y in 0..size {
        for x in 0..size {
            let h = terrain[y][x];
            let norm = if (max - min).abs() < f32::EPSILON {
                0.5
            } else {
                (h - min) / (max - min)
            };
            // Base color from gradient
            let col: LinSrgb = gradient.get(norm as f32);
            let rgb = col.into_format::<u8>();
            // Apply hillshade
            let light = (shade[y][x] * 0.5 + 0.5).clamp(0.0, 1.0);
            let pixel = Rgb([
                (rgb.red as f32 * light) as u8,
                (rgb.green as f32 * light) as u8,
                (rgb.blue as f32 * light) as u8,
            ]);
            img.put_pixel(x as u32, y as u32, pixel);
        }
    }

    // Save
    let path = Path::new("terrain_final.png");
    img.save(path).unwrap();
    println!("Saved final terrain image to {:?}", path);
}
