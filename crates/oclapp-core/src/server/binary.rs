use crate::error::Result;
use std::path::PathBuf;

pub async fn ensure_binary() -> Result<PathBuf> {
    // TODO: Detect platform, download llama.cpp server binary if missing
    Ok(PathBuf::from("llama-server"))
}
