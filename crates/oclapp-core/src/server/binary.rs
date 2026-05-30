use crate::error::{OclappError, Result};
use std::path::{Path, PathBuf};

const LLAMA_RELEASE_URL: &str = "https://github.com/ggml-org/llama.cpp/releases/latest/download";

/// Which llama.cpp distribution was found.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryFlavor {
    /// New llama.app unified binary (`llama serve`).
    LlamaApp,
    /// Legacy llama.cpp server binary (`llama-server`).
    Legacy,
}

#[derive(Debug, Clone)]
pub struct ResolvedBinary {
    pub path: PathBuf,
    pub flavor: BinaryFlavor,
}

pub fn detect_platform() -> (String, String) {
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

    let binary_name = if cfg!(target_os = "windows") {
        "llama-server.exe"
    } else {
        "llama-server"
    };

    let download_url = format!("{}/{}", LLAMA_RELEASE_URL, target);
    (binary_name.to_string(), download_url)
}

/// Search well-known install locations that macOS app bundles don't inherit via PATH.
pub fn well_known_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    if let Some(home) = dirs::home_dir() {
        dirs.push(home.join(".llama-app"));
        dirs.push(home.join(".local/bin"));
    }

    if cfg!(target_os = "macos") {
        dirs.push(PathBuf::from("/opt/homebrew/bin"));
        dirs.push(PathBuf::from("/usr/local/bin"));
    }

    if cfg!(target_os = "linux") {
        dirs.push(PathBuf::from("/usr/local/bin"));
        dirs.push(PathBuf::from("/usr/bin"));
    }

    dirs
}

fn find_in_path(binary_name: &str) -> Option<PathBuf> {
    if let Ok(path_env) = std::env::var("PATH") {
        let sep = if cfg!(target_os = "windows") { ';' } else { ':' };
        for dir in path_env.split(sep) {
            let candidate = Path::new(dir).join(binary_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

pub fn find_binary(name: &str) -> Option<PathBuf> {
    // Check PATH first
    if let Some(p) = find_in_path(name) {
        return Some(p);
    }
    // Check well-known directories
    for dir in well_known_dirs() {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Run the llama.app install script.
async fn run_install_script() -> Result<PathBuf> {
    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg("curl -LsSf https://llama.app/install.sh | sh")
        .output()
        .await
        .map_err(|e| OclappError::Validation(format!("Failed to run install script: {}", e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(OclappError::Validation(format!(
            "llama.app install script failed:\n{}",
            stderr
        )));
    }

    // After install, check again
    if let Some(p) = find_binary("llama") {
        return Ok(p);
    }

    if let Some(home) = dirs::home_dir() {
        let fallback = home.join(".llama-app/llama");
        if fallback.is_file() {
            return Ok(fallback);
        }
    }

    Err(OclappError::Validation(
        "llama binary not found after running install script".to_string(),
    ))
}

pub async fn ensure_binary(data_dir: &Path) -> Result<ResolvedBinary> {
    // 1. New llama.app unified binary (`llama serve`)
    if let Some(path) = find_binary("llama") {
        return Ok(ResolvedBinary {
            path,
            flavor: BinaryFlavor::LlamaApp,
        });
    }

    let bin_dir = data_dir.join("bin");
    tokio::fs::create_dir_all(&bin_dir).await.map_err(OclappError::Io)?;

    // 2. Legacy cached binary
    let (legacy_name, download_url) = detect_platform();
    let legacy_path = bin_dir.join(&legacy_name);
    if legacy_path.exists() {
        return Ok(ResolvedBinary {
            path: legacy_path,
            flavor: BinaryFlavor::Legacy,
        });
    }

    // 3. Try to auto-install llama.app
    if let Ok(path) = run_install_script().await {
        return Ok(ResolvedBinary {
            path,
            flavor: BinaryFlavor::LlamaApp,
        });
    }

    // 4. Fallback: download legacy release
    let client = reqwest::Client::new();
    let response = client
        .get(&download_url)
        .send()
        .await
        .map_err(OclappError::Network)?;

    if !response.status().is_success() {
        return Err(OclappError::Validation(format!(
            "Failed to download llama.cpp binary (HTTP {}). \
            Please install llama.cpp manually:\n\
            • macOS: brew install llama.cpp  OR  curl -LsSf https://llama.app/install.sh | sh\n\
            • Linux: curl -LsSf https://llama.app/install.sh | sh\n\
            • Windows: winget install llama.cpp\n\
            More info: https://llama.app/",
            response.status()
        )));
    }

    let zip_path = bin_dir.join("llama.zip");
    let bytes = response.bytes().await.map_err(OclappError::Network)?;
    tokio::fs::write(&zip_path, &bytes)
        .await
        .map_err(OclappError::Io)?;

    // Extract zip in a blocking task because zip::ZipFile contains non-Send dyn Read
    let bin_dir_clone = bin_dir.clone();
    let zip_path_clone = zip_path.clone();
    tokio::task::spawn_blocking(move || {
        let archive = std::fs::File::open(&zip_path_clone).map_err(OclappError::Io)?;
        let mut zip = zip::ZipArchive::new(archive).map_err(|e| {
            OclappError::Validation(format!("Failed to read zip archive: {}", e))
        })?;

        for i in 0..zip.len() {
            let mut file = zip.by_index(i).map_err(|e| {
                OclappError::Validation(format!("Failed to read zip entry: {}", e))
            })?;

            let outpath = bin_dir_clone.join(file.name());
            if file.name().ends_with('/') {
                std::fs::create_dir_all(&outpath).map_err(OclappError::Io)?;
            } else {
                let mut outfile = std::fs::File::create(&outpath).map_err(OclappError::Io)?;
                let mut buf = Vec::new();
                std::io::Read::read_to_end(&mut file, &mut buf).map_err(|e| {
                    OclappError::Validation(format!("Failed to read zip data: {}", e))
                })?;
                std::io::Write::write_all(&mut outfile, &buf).map_err(OclappError::Io)?;

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mut perms = std::fs::metadata(&outpath)
                        .map_err(OclappError::Io)?
                        .permissions();
                    perms.set_mode(0o755);
                    std::fs::set_permissions(&outpath, perms).map_err(OclappError::Io)?;
                }
            }
        }

        Ok::<_, crate::error::OclappError>(())
    })
    .await
    .map_err(|e| OclappError::Validation(format!("Zip extraction task failed: {}", e)))??;

    let _ = tokio::fs::remove_file(&zip_path).await;

    if legacy_path.exists() {
        return Ok(ResolvedBinary {
            path: legacy_path,
            flavor: BinaryFlavor::Legacy,
        });
    }

    Err(OclappError::Validation(
        "llama-server binary not found after extraction".to_string(),
    ))
}
