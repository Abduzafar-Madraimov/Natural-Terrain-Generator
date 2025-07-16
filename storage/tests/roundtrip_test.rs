#[test]
fn test_roundtrip_2d() {
    // Bring things into scope
    use core::{Fractal2D, ThermalErosion2D, utils::flatten2};
    use storage::Storage2D;
    use storage::models::{TerrainDoc2D, TerrainParams};
    use tokio::runtime::Builder;

    // Build a single-threaded Tokio runtime
    let rt = Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build Tokio runtime");

    // Run async workflow inside it
    rt.block_on(async {
        // Generate a small height‐map
        let size = 65;
        let mut grid = Fractal2D::new(size, 42, 1.0).generate();
        ThermalErosion2D::new(3, 1.0).apply(&mut grid);
        let flat = flatten2(&grid);

        // Prepare the document
        let params = TerrainParams {
            noise_type: "fractal2d".to_string(),
            frequency: 0.0,
            persistence: 0.0,
            octaves: 0,
            roughness: Some(1.0),
            erosion_iters: Some(3),
            talus_angle: Some(1.0),
        };
        let doc = TerrainDoc2D {
            id: None,
            seed: 42,
            params,
            height_map: flat.clone(),
            dimensions: 2,
        };

        // Initialize storage (MongoDB must be running)
        let storage = Storage2D::init("mongodb://localhost:27017", "terrain_db", "terrain2d")
            .await
            .expect("storage init failed");

        // Insert, read back, assert
        storage.create(doc).await.expect("create failed");
        let found = storage
            .read_by_seed(42)
            .await
            .expect("read failed")
            .expect("doc not found");

        assert_eq!(found.height_map.len(), size * size);
        assert_eq!(found.height_map[size * size / 2], flat[size * size / 2]);

        // Clean up
        storage.delete_by_seed(42).await.expect("delete failed");
    });
}
