use crate::error::{OclappError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub author: String,
    pub tags: Vec<String>,
    pub downloads: u64,
    pub likes: u64,
    pub last_modified: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalModel {
    pub id: String,
    pub path: String,
    pub size: u64,
    pub format: ModelFormat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModelFormat {
    Gguf,
    Mlx,
    Unknown,
}

impl std::fmt::Display for ModelFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelFormat::Gguf => write!(f, "GGUF"),
            ModelFormat::Mlx => write!(f, "MLX"),
            ModelFormat::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Deserialize)]
struct HfModelItem {
    id: String,
    author: Option<String>,
    tags: Option<Vec<String>>,
    downloads: Option<u64>,
    likes: Option<u64>,
    last_modified: Option<String>,
}

pub async fn search_huggingface(query: &str) -> Result<Vec<ModelInfo>> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://huggingface.co/api/models?search={}&limit=20&sort=downloads",
        urlencoding::encode(query)
    );

    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(|e| OclappError::Network(e))?;

    if !response.status().is_success() {
        return Err(OclappError::Validation(format!(
            "HuggingFace API returned status {}",
            response.status()
        )));
    }

    let items: Vec<HfModelItem> = response
        .json()
        .await
        .map_err(|e| OclappError::Network(e))?;

    let models = items
        .into_iter()
        .map(|item| ModelInfo {
            id: item.id.clone(),
            name: item.id.split('/').last().unwrap_or(&item.id).to_string(),
            author: item.author.unwrap_or_default(),
            tags: item.tags.unwrap_or_default(),
            downloads: item.downloads.unwrap_or(0),
            likes: item.likes.unwrap_or(0),
            last_modified: item.last_modified.unwrap_or_default(),
        })
        .collect();

    Ok(models)
}

pub fn list_local_models(models_dir: &Path) -> Result<Vec<LocalModel>> {
    let mut models = Vec::new();

    if !models_dir.exists() {
        return Ok(models);
    }

    for entry in std::fs::read_dir(models_dir).map_err(OclappError::Io)? {
        let entry = entry.map_err(OclappError::Io)?;
        let path = entry.path();

        if path.is_file() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();

            let format = match ext.as_str() {
                "gguf" => ModelFormat::Gguf,
                "mlx" => ModelFormat::Mlx,
                _ => ModelFormat::Unknown,
            };

            if matches!(format, ModelFormat::Gguf | ModelFormat::Mlx) {
                let size = entry.metadata().map_err(OclappError::Io)?.len();
                let id = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();

                models.push(LocalModel {
                    id,
                    path: path.to_string_lossy().to_string(),
                    size,
                    format,
                });
            }
        }
    }

    Ok(models)
}

pub fn resolve_model_path(models_dir: &Path, model_name: &str) -> Option<std::path::PathBuf> {
    let gguf_path = models_dir.join(format!("{}.gguf", model_name));
    if gguf_path.exists() {
        return Some(gguf_path);
    }

    let mlx_path = models_dir.join(format!("{}.mlx", model_name));
    if mlx_path.exists() {
        return Some(mlx_path);
    }

    // Try exact match if user provided full filename
    let exact_path = models_dir.join(model_name);
    if exact_path.exists() {
        return Some(exact_path);
    }

    None
}
