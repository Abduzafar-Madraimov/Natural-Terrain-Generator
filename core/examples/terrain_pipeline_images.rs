// Generates and saves three 257×257 terrain images:
// Base Perlin2D
// Diamond–Square Fractal
// Fractal + Thermal Erosion

use core::utils::flatten2;
use core::{Fractal2D, NoiseGenerator, Perlin2D, ThermalErosion2D};
use image::{GrayImage, Luma};
use std::path::Path;

fn save_grayscale(grid: &[Vec<f32>], filename: &str) {
    let size = grid.len();
    // Find min/max
    let mut min = f32::INFINITY;
    let mut max = f32::NEG_INFINITY;
    for row in grid {
        for &v in row {
            min = min.min(v);
            max = max.max(v);
        }
    }
    let mut img = GrayImage::new(size as u32, size as u32);
    for y in 0..size {
        for x in 0..size {
            let v = grid[y][x];
            let norm = if (max - min).abs() < f32::EPSILON {
                0.5
            } else {
                (v - min) / (max - min)
            };
            let gray = (norm * 255.0).round() as u8;
            img.put_pixel(x as u32, y as u32, Luma([gray]));
        }
    }
    img.save(Path::new(filename)).unwrap();
    println!("Saved {}", filename);
}

fn main() {
    let size = 257;
    let scale = 4.0;
    let persistence = 0.5;
    let octaves = 4;

    // 1) Base Perlin2D
    let perlin = Perlin2D::new(42, scale, persistence, octaves);
    let mut perlin_grid = vec![vec![0.0f32; size]; size];
    for y in 0..size {
        for x in 0..size {
            perlin_grid[y][x] = perlin.get2(x as f64 / size as f64, y as f64 / size as f64) as f32;
        }
    }
    save_grayscale(&perlin_grid, "terrain_perlin2d.png");

    // 2) Diamond–Square Fractal
    let fractal = Fractal2D::new(size, 2025, 1.0);
    let fractal_grid = fractal.generate();
    save_grayscale(&fractal_grid, "terrain_fractal2d.png");

    // 3) Fractal + Thermal Erosion
    let mut eroded = fractal_grid.clone();
    let erosion = ThermalErosion2D::new(10, 1.0);
    erosion.apply(&mut eroded);
    save_grayscale(&eroded, "terrain_fractal2d_eroded.png");
}
