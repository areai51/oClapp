use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub size: u64,
    pub quantization: Option<String>,
    pub downloads: u64,
    pub last_updated: String,
}

pub async fn search_models(_query: &str) -> Result<Vec<ModelInfo>> {
    // TODO: Implement HuggingFace search
    Ok(vec![])
}
