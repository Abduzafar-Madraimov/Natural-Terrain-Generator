use core::{Fractal2D, ThermalErosion2D, utils::flatten2};
use storage::Storage2D;
use storage::models::{TerrainDoc2D, TerrainParams};
use tokio;

#[tokio::main]
async fn main() -> mongodb::error::Result<()> {
    // Generate a 257×257 fractal + erosion
    let size = 257;
    let mut fractal = Fractal2D::new(size, 2025, 1.0);
    let mut map2 = fractal.generate();
    let erosion = ThermalErosion2D::new(10, 1.0);
    erosion.apply(&mut map2);

    // Flatten
    let flat = flatten2(&map2);

    // Build the document
    let params = TerrainParams {
        noise_type: "fractal2d".to_string(),
        frequency: 0.0,
        persistence: 0.0,
        octaves: 0,
        roughness: Some(1.0),
        erosion_iters: Some(10),
        talus_angle: Some(1.0),
    };
    let doc = TerrainDoc2D {
        id: None,
        seed: 2025,
        params,
        height_map: flat.clone(),
        dimensions: 2,
    };

    // Init storage
    let storage = Storage2D::init("mongodb://localhost:27017", "terrain_db", "terrain2d").await?;

    // Insert & read back
    storage.create(doc).await?;
    if let Some(found) = storage.read_by_seed(2025).await? {
        println!(
            "Round-trip success: sample [128,128] = {}",
            found.height_map[128 * size + 128]
        );
    } else {
        println!("Document not found!");
    }

    // Clean up
    storage.delete_by_seed(2025).await?;

    Ok(())
}
