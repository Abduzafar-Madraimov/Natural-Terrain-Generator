// bench holds Criterion benchmarks (binary crate)

use core::{Fractal2D, NoiseGenerator, Perlin2D, Perlin3D, Simplex2D, ThermalErosion2D};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

// Benchmark a single Perlin2D point sample
fn bench_perlin2_point(c: &mut Criterion) {
    let p2 = Perlin2D::new(42, 4.0, 0.5, 4);
    c.bench_function("Perlin2D single point", |b| {
        b.iter(|| black_box(p2.get2(0.123, 0.456)))
    });
}

// Benchmark a single Simplex2D point sample
fn bench_simplex2_point(c: &mut Criterion) {
    let s2 = Simplex2D::new(42, 4.0, 0.5, 4);
    c.bench_function("Simplex2D single point", |b| {
        b.iter(|| black_box(s2.get2(0.123, 0.456)))
    });
}

// Benchmark a single Perlin3D point sample
fn bench_perlin3_point(c: &mut Criterion) {
    let p3 = Perlin3D::new(42, 4.0, 0.5, 4);
    c.bench_function("Perlin3D single point", |b| {
        b.iter(|| black_box(p3.get3(0.123, 0.456, 0.789)))
    });
}

/// Benchmark a full 257×257 fractal generation
fn bench_fractal2_full(c: &mut Criterion) {
    c.bench_function("Fractal2D 257×257", |b| {
        b.iter(|| {
            let f = Fractal2D::new(257, 2025, 1.0);
            black_box(f.generate());
        })
    });
}

// Benchmark fractal + 10× thermal erosion on 257×257
fn bench_fractal2_plus_erosion(c: &mut Criterion) {
    // Pre-generate a base map to avoid timing its creation each iter
    let base = Fractal2D::new(257, 2025, 1.0).generate();
    let er = ThermalErosion2D::new(10, 1.0);
    c.bench_function("Fractal2D + erosion (10 iters)", |b| {
        b.iter(|| {
            let mut clone = base.clone();
            er.apply(&mut clone);
            black_box(clone);
        })
    });
}

criterion_group!(
    benches,
    bench_perlin2_point,
    bench_simplex2_point,
    bench_perlin3_point,
    bench_fractal2_full,
    bench_fractal2_plus_erosion
);
criterion_main!(benches);
