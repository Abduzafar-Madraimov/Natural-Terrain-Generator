use core::{
    DomainWarp2D, Fractal2D, NoiseGenerator, Perlin2D, Simplex2D, ThermalErosion2D,
    utils::{HeightMap2D, flatten2, normalize2, to_terrain_image},
};
use criterion::{Criterion, criterion_group, criterion_main};

const SIZE: usize = 257;
const SEED: u64 = 2025;

fn bench_fractal_pipeline(c: &mut Criterion) {
    c.bench_function("Fractal2D + normalize + flatten + image", |b| {
        b.iter(|| {
            let mut f = Fractal2D::new(SIZE, SEED, 1.0);
            let mut map = f.generate();
            normalize2(&mut map);
            let flat = flatten2(&map);
            let _img = to_terrain_image(&flat, SIZE);
        })
    });
}

fn bench_fractal_with_erosion(c: &mut Criterion) {
    c.bench_function(
        "Fractal2D + erosion (5 iters) + normalize + flatten + image",
        |b| {
            b.iter(|| {
                let mut f = Fractal2D::new(SIZE, SEED, 1.0);
                let mut map = f.generate();
                ThermalErosion2D::new(5, 1.0).apply(&mut map);
                normalize2(&mut map);
                let flat = flatten2(&map);
                let _img = to_terrain_image(&flat, SIZE);
            })
        },
    );
}

fn bench_perlin2_plain(c: &mut Criterion) {
    c.bench_function("Perlin2D plain + normalize + flatten + image", |b| {
        b.iter(|| {
            let perlin = Perlin2D::new(SEED, 4.0, 0.5, 4);
            let mut map: HeightMap2D = (0..SIZE)
                .map(|y| {
                    (0..SIZE)
                        .map(|x| perlin.get2(x as f64 / SIZE as f64, y as f64 / SIZE as f64) as f32)
                        .collect()
                })
                .collect();
            normalize2(&mut map);
            let flat = flatten2(&map);
            let _img = to_terrain_image(&flat, SIZE);
        })
    });
}

fn bench_perlin_with_warp(c: &mut Criterion) {
    c.bench_function(
        "Perlin2D + Domain Warp + normalize + flatten + image",
        |b| {
            b.iter(|| {
                let base = Perlin2D::new(SEED, 4.0, 0.5, 4);
                let warp = Perlin2D::new(SEED.wrapping_add(42), 4.0, 0.5, 4);
                let mut map = DomainWarp2D {
                    base: &base,
                    warp: &warp,
                    size: SIZE,
                    warp_strength: 0.5,
                }
                .generate();
                normalize2(&mut map);
                let flat = flatten2(&map);
                let _img = to_terrain_image(&flat, SIZE);
            })
        },
    );
}

fn bench_simplex_plain(c: &mut Criterion) {
    c.bench_function("Simplex2D generate + normalize + flatten + image", |b| {
        b.iter(|| {
            let simplex = Simplex2D::new(SEED, 4.0, 0.5, 4);
            let mut map: HeightMap2D = (0..SIZE)
                .map(|y| {
                    (0..SIZE)
                        .map(|x| {
                            simplex.get2(x as f64 / SIZE as f64, y as f64 / SIZE as f64) as f32
                        })
                        .collect()
                })
                .collect();
            normalize2(&mut map);
            let flat = flatten2(&map);
            let _img = to_terrain_image(&flat, SIZE);
        })
    });
}

fn bench_simplex_with_warp(c: &mut Criterion) {
    c.bench_function(
        "Simplex2D + Domain Warp + normalize + flatten + image",
        |b| {
            b.iter(|| {
                let base = Simplex2D::new(SEED, 4.0, 0.5, 4);
                let warp = Simplex2D::new(SEED.wrapping_add(42), 4.0, 0.5, 4);
                let mut map = DomainWarp2D {
                    base: &base,
                    warp: &warp,
                    size: SIZE,
                    warp_strength: 0.5,
                }
                .generate();
                normalize2(&mut map);
                let flat = flatten2(&map);
                let _img = to_terrain_image(&flat, SIZE);
            })
        },
    );
}

criterion_group!(
    terrain_benchmarks,
    bench_fractal_pipeline,
    bench_fractal_with_erosion,
    bench_perlin2_plain,
    bench_perlin_with_warp,
    bench_simplex_plain,
    bench_simplex_with_warp
);
criterion_main!(terrain_benchmarks);
