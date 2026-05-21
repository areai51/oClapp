use crate::error::{OclappError, Result};
use std::path::PathBuf;

const LLAMA_RELEASE_URL: &str = "https://github.com/ggml-org/llama.cpp/releases/latest/download";

#[derive(Debug, Clone)]
pub struct PlatformBinary {
    pub binary_name: String,
    pub download_url: String,
}

pub fn detect_platform() -> PlatformBinary {
    let target = if cfg!(target_os = "macos") {
        if cfg!(target_arch = "aarch64") {
            "llama-b4744-bin-macos-arm64.zip"
        } else {
            "llama-b4744-bin-macos-x64.zip"
        }
    } else if cfg!(target_os = "linux") {
        if cfg!(target_arch = "aarch64") {
            "llama-b4744-bin-ubuntu-arm64.zip"
        } else {
            "llama-b4744-bin-ubuntu-x64.zip"
        }
    } else if cfg!(target_os = "windows") {
        if cfg!(target_arch = "aarch64") {
            "llama-b4744-bin-win-arm64.zip"
        } else {
            "llama-b4744-bin-win-x64.zip"
        }
    } else {
        "llama-b4744-bin-ubuntu-x64.zip"
    };

    PlatformBinary {
        binary_name: if cfg!(target_os = "windows") {
            "llama-server.exe".to_string()
        } else {
            "llama-server".to_string()
        },
        download_url: format!("{}/{}", LLAMA_RELEASE_URL, target),
    }
}

pub async fn ensure_binary(data_dir: &std::path::Path) -> Result<PathBuf> {
    let bin_dir = data_dir.join("bin");
    tokio::fs::create_dir_all(&bin_dir).await.map_err(OclappError::Io)?;

    let platform = detect_platform();
    let binary_path = bin_dir.join(&platform.binary_name);

    if binary_path.exists() {
        return Ok(binary_path);
    }

    // Download binary
    let client = reqwest::Client::new();
    let response = client
        .get(&platform.download_url)
        .send()
        .await
        .map_err(OclappError::Network)?;

    if !response.status().is_success() {
        return Err(OclappError::Validation(format!(
            "Failed to download llama.cpp binary: HTTP {}",
            response.status()
        )));
    }

    let zip_path = bin_dir.join("llama.zip");
    let bytes = response.bytes().await.map_err(OclappError::Network)?;
    tokio::fs::write(&zip_path, &bytes)
        .await
        .map_err(OclappError::Io)?;

    // Extract zip
    let archive = std::fs::File::open(&zip_path).map_err(OclappError::Io)?;
    let mut zip = zip::ZipArchive::new(archive).map_err(|e| {
        OclappError::Validation(format!("Failed to read zip archive: {}", e))
    })?;

    for i in 0..zip.len() {
        let mut file = zip.by_index(i).map_err(|e| {
            OclappError::Validation(format!("Failed to read zip entry: {}", e))
        })?;

        let outpath = bin_dir.join(file.name());
        if file.name().ends_with('/') {
            tokio::fs::create_dir_all(&outpath)
                .await
                .map_err(OclappError::Io)?;
        } else {
            let mut outfile = tokio::fs::File::create(&outpath)
                .await
                .map_err(OclappError::Io)?;
            let mut buf = Vec::new();
            std::io::Read::read_to_end(&mut file, &mut buf).map_err(|e| {
                OclappError::Validation(format!("Failed to read zip data: {}", e))
            })?;
            tokio::io::AsyncWriteExt::write_all(&mut outfile, &buf)
                .await
                .map_err(OclappError::Io)?;

            // Make executable on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = tokio::fs::metadata(&outpath)
                    .await
                    .map_err(OclappError::Io)?
                    .permissions();
                perms.set_mode(0o755);
                tokio::fs::set_permissions(&outpath, perms)
                    .await
                    .map_err(OclappError::Io)?;
            }
        }
    }

    // Clean up zip
    let _ = tokio::fs::remove_file(&zip_path).await;

    if !binary_path.exists() {
        return Err(OclappError::Validation(
            "llama-server binary not found after extraction".to_string(),
        ));
    }

    Ok(binary_path)
}
