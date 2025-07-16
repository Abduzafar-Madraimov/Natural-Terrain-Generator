use core::{NoiseGenerator, Perlin2D, Perlin3D, Simplex2D};
use image::{GrayImage, Luma};
use std::path::Path;

fn save_noise2d<N: NoiseGenerator>(generator: N, size: usize, filename: &str)
where
    N: NoiseGenerator,
{
    let mut img = GrayImage::new(size as u32, size as u32);
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut data = vec![vec![0.0f64; size]; size];

    // Sample noise
    for y in 0..size {
        for x in 0..size {
            let v = generator.get2(x as f64 / size as f64, y as f64 / size as f64);
            data[y][x] = v;
            min = min.min(v);
            max = max.max(v);
        }
    }

    // Write image
    for y in 0..size {
        for x in 0..size {
            let v = data[y][x];
            let norm = if (max - min).abs() < f64::EPSILON {
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

fn save_perlin3d_slice(size: usize, slice_z: usize, filename: &str) {
    let generator = Perlin3D::new(42, 4.0, 0.5, 4);
    let mut img = GrayImage::new(size as u32, size as u32);
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut data = vec![vec![0.0f64; size]; size];

    for y in 0..size {
        for x in 0..size {
            let v = generator.get3(
                x as f64 / size as f64,
                y as f64 / size as f64,
                slice_z as f64 / size as f64,
            );
            data[y][x] = v;
            min = min.min(v);
            max = max.max(v);
        }
    }
    for y in 0..size {
        for x in 0..size {
            let v = data[y][x];
            let norm = if (max - min).abs() < f64::EPSILON {
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
    let size = 256;
    // Generate and save 2D Perlin
    let perlin2 = Perlin2D::new(42, 4.0, 0.5, 4);
    save_noise2d(perlin2, size, "perlin2d.png");

    // 2D Simplex
    let simplex = Simplex2D::new(42, 4.0, 0.5, 4);
    save_noise2d(simplex, size, "simplex2d.png");

    // 3D Perlin slice at z = 128
    save_perlin3d_slice(size, size / 2, "perlin3d_slice.png");
}
