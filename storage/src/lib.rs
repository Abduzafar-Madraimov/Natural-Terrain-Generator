//storage holds MongoDB schema & async CRUD

pub mod models;

use crate::models::{TerrainDoc2D, TerrainParams};
use bson::{Bson, doc};
use futures_util::stream::TryStreamExt;
use mongodb::{Client, Collection, IndexModel, options::ClientOptions};

pub struct Storage2D {
    col: Collection<TerrainDoc2D>,
}

impl Storage2D {
    // Initialize the MongoDB collection
    pub async fn init(uri: &str, db_name: &str, col_name: &str) -> mongodb::error::Result<Self> {
        let mut opts = ClientOptions::parse(uri).await?;
        opts.app_name = Some("FYPStorage".to_string());
        let client = Client::with_options(opts)?;
        let col = client.database(db_name).collection(col_name);

        // Create unique index on (name, seed, dimensions)
        let index_model = mongodb::IndexModel::builder()
            .keys(doc! { "name": 1, "seed": 1, "dimensions": 1 })
            .options(None)
            .build();
        col.create_index(index_model).await?;

        Ok(Self { col })
    }

    pub async fn list_names(&self) -> mongodb::error::Result<Vec<String>> {
        let mut cursor = self.col.find(doc! { "dimensions": 2i32 }).await?;
        let mut names = Vec::new();
        while let Some(doc) = cursor.try_next().await? {
            names.push(doc.name);
        }
        Ok(names)
    }

    pub async fn read_by_name(&self, name: &str) -> mongodb::error::Result<Option<TerrainDoc2D>> {
        self.col
            .find_one(doc! { "name": name, "dimensions": 2i32 })
            .await
    }

    // Insert a terrain document.
    pub async fn create(&self, doc_obj: TerrainDoc2D) -> mongodb::error::Result<()> {
        // Delete any existing document with same seed+dimensions
        let filter = doc! {
            "name": &doc_obj.name,
            "seed": doc_obj.seed,
            "dimensions": i32::from(doc_obj.dimensions),
        };
        let _ = self.col.delete_one(filter.clone()).await;

        // Insert the new document
        self.col.insert_one(doc_obj).await?;
        Ok(())
    }

    // Read a terrain by seed.
    pub async fn read_by_seed(&self, seed: i64) -> mongodb::error::Result<Option<TerrainDoc2D>> {
        self.col
            .find_one(doc! { "seed": seed, "dimensions": 2i32 })
            .await
    }

    // Delete by seed (for clean-up).
    pub async fn delete_by_seed(&self, seed: i64) -> mongodb::error::Result<()> {
        let filter = doc! {
            "seed": seed,
            "dimensions": 2i32,
        };
        self.col.delete_one(filter).await?;
        Ok(())
    }
}
