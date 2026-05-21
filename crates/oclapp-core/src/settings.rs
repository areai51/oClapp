use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub models_dir: PathBuf,
    pub server_port: u16,
    pub curated_params: CuratedParams,
    pub advanced_params: AdvancedParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuratedParams {
    pub temperature: f32,
    pub top_p: f32,
    pub context_size: u32,
    pub max_tokens: u32,
    pub gpu_layers: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedParams {
    pub seed: u32,
    pub repeat_penalty: f32,
    pub frequency_penalty: f32,
    pub presence_penalty: f32,
    pub batch_size: u32,
    pub threads: u32,
    pub flash_attention: bool,
    pub mmap: bool,
    pub mlock: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            models_dir: dirs::home_dir().unwrap_or_default().join(".oclapp/models"),
            server_port: 8080,
            curated_params: CuratedParams {
                temperature: 0.7,
                top_p: 0.9,
                context_size: 4096,
                max_tokens: 2048,
                gpu_layers: -1,
            },
            advanced_params: AdvancedParams {
                seed: 0,
                repeat_penalty: 1.1,
                frequency_penalty: 0.0,
                presence_penalty: 0.0,
                batch_size: 512,
                threads: 4,
                flash_attention: false,
                mmap: true,
                mlock: false,
            },
        }
    }
}

pub async fn load_settings() -> Result<Settings> {
    // TODO: Implement settings loading via tauri-plugin-store or file
    Ok(Settings::default())
}

pub async fn save_settings(_settings: &Settings) -> Result<()> {
    // TODO: Implement settings persistence
    Ok(())
}
