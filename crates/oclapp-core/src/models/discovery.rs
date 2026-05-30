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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ModelFormat {
    Gguf,
    Mlx,
    Safetensors,
    Unknown,
}

impl std::fmt::Display for ModelFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModelFormat::Gguf => write!(f, "GGUF"),
            ModelFormat::Mlx => write!(f, "MLX"),
            ModelFormat::Safetensors => write!(f, "Safetensors"),
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

fn is_huggingface_model_dir(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    let has_config = path.join("config.json").exists();
    let has_safetensors = std::fs::read_dir(path)
        .ok()
        .map(|entries| {
            entries.filter_map(|e| e.ok()).any(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("safetensors"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false);
    has_config || has_safetensors
}

fn dir_size(path: &Path) -> Result<u64> {
    let mut total = 0u64;
    for entry in std::fs::read_dir(path).map_err(OclappError::Io)? {
        let entry = entry.map_err(OclappError::Io)?;
        let meta = entry.metadata().map_err(OclappError::Io)?;
        if meta.is_file() {
            total += meta.len();
        } else if meta.is_dir() {
            total += dir_size(&entry.path())?;
        }
    }
    Ok(total)
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
                "safetensors" => ModelFormat::Safetensors,
                _ => ModelFormat::Unknown,
            };

            if matches!(format, ModelFormat::Gguf | ModelFormat::Mlx | ModelFormat::Safetensors) {
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
        } else if is_huggingface_model_dir(&path) {
            let size = dir_size(&path)?;
            let id = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();

            models.push(LocalModel {
                id,
                path: path.to_string_lossy().to_string(),
                size,
                format: ModelFormat::Safetensors,
            });
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

    let safetensors_path = models_dir.join(format!("{}.safetensors", model_name));
    if safetensors_path.exists() {
        return Some(safetensors_path);
    }

    // HuggingFace model directory
    let hf_dir = models_dir.join(model_name);
    if hf_dir.is_dir() && is_huggingface_model_dir(&hf_dir) {
        return Some(hf_dir);
    }

    // Try exact match if user provided full filename
    let exact_path = models_dir.join(model_name);
    if exact_path.exists() {
        return Some(exact_path);
    }

    None
}
