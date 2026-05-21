use crate::error::{OclappError, Result};
use futures_util::StreamExt;
use hf_hub::api::tokio::Api;
use std::path::Path;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone)]
pub struct DownloadProgress {
    pub total_bytes: u64,
    pub downloaded_bytes: u64,
}

pub async fn download_from_hf(
    repo_id: &str,
    filename: &str,
    target_dir: &Path,
    _progress_callback: Option<&(dyn Fn(DownloadProgress) + Send + Sync)>,
) -> Result<std::path::PathBuf> {
    let api = Api::new().map_err(|e| {
        OclappError::Validation(format!("Failed to initialize HF API: {}", e))
    })?;

    let repo = api.model(repo_id.to_string());

    let file_path = repo.get(filename).await.map_err(|e| {
        OclappError::Validation(format!(
            "Failed to download {} from {}: {}",
            filename, repo_id, e
        ))
    })?;

    // Copy from hf-hub cache to target directory
    let target_path = target_dir.join(filename);
    tokio::fs::copy(&file_path, &target_path)
        .await
        .map_err(OclappError::Io)?;

    Ok(target_path)
}

pub async fn download_file_with_progress(
    url: &str,
    target_path: &Path,
    progress_callback: Option<&(dyn Fn(DownloadProgress) + Send + Sync)>,
) -> Result<()> {
    let client = reqwest::Client::new();

    // Check if partial file exists and get its size for resume
    let partial_size = if target_path.exists() {
        tokio::fs::metadata(target_path)
            .await
            .map_err(OclappError::Io)?
            .len()
    } else {
        0
    };

    let mut request = client.get(url);

    if partial_size > 0 {
        request = request.header("Range", format!("bytes={}-", partial_size));
    }

    let response = request.send().await.map_err(OclappError::Network)?;

    let status = response.status();
    if !status.is_success() && status != reqwest::StatusCode::PARTIAL_CONTENT {
        return Err(OclappError::Validation(format!(
            "HTTP error during download: {}",
            status
        )));
    }

    let total_size = response
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(0);

    let mut file = if partial_size > 0 && status == reqwest::StatusCode::PARTIAL_CONTENT {
        tokio::fs::OpenOptions::new()
            .append(true)
            .open(target_path)
            .await
            .map_err(OclappError::Io)?
    } else {
        tokio::fs::File::create(target_path)
            .await
            .map_err(OclappError::Io)?
    };

    let mut stream = response.bytes_stream();
    let mut downloaded = partial_size;

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(OclappError::Network)?;
        file.write_all(&chunk).await.map_err(OclappError::Io)?;
        downloaded += chunk.len() as u64;

        if let Some(cb) = progress_callback {
            cb(DownloadProgress {
                total_bytes: total_size + partial_size,
                downloaded_bytes: downloaded,
            });
        }
    }

    file.flush().await.map_err(OclappError::Io)?;

    Ok(())
}
