use core::Fractal2D;
use core::ThermalErosion2D;
use core::flatten2;

fn main() {
    // Generate a 129×129 fractal with seed 2025, roughness 1.0
    let mut fractal = Fractal2D::new(129, 2025, 1.0);
    let mut map = fractal.generate();

    // Apply 5 iterations of thermal erosion with talus_angle = 1.0
    let erosion = ThermalErosion2D::new(5, 1.0);
    erosion.apply(&mut map);

    // Print the top-left 16×16 corner of the map
    for y in 0..16 {
        for x in 0..16 {
            print!("{:>6.3} ", map[y][x]);
        }
        println!();
    }
}
