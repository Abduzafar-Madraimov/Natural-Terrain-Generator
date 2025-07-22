use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct TerrainParams {
    pub noise_type: String, // e.g. "perlin2d", "fractal2d"
    pub frequency: f64,
    pub persistence: f64,
    pub octaves: usize,
    pub roughness: Option<f64>, // for fractal
    pub erosion_iters: Option<u32>,
    pub talus_angle: Option<f32>,
    pub warp_strength: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TerrainDoc2D {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none", default)]
    pub id: Option<ObjectId>,
    pub name: String,
    pub seed: i64,
    pub params: TerrainParams,
    // Flattened row-major: length = size×size
    pub height_map: Vec<f32>,
    pub dimensions: u8, // should always be 2 here
}
